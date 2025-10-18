use std::sync::mpsc::channel;

use derive_builder::Builder;
use image::RgbaImage;
use taffy::{AvailableSpace, Layout, NodeId, Point, TaffyTree, geometry::Size};

use crate::{
  GlobalContext,
  layout::{
    Viewport,
    node::Node,
    style::{Affine, InheritedStyle},
    tree::NodeTree,
  },
  rendering::{
    Canvas, create_blocking_canvas_loop, draw_debug_border, inline_drawing::draw_inline_layout,
  },
};

use crate::rendering::RenderContext;

#[derive(Clone, Builder)]
/// Options for rendering a node. Construct using [`RenderOptionsBuilder`] to avoid breaking changes.
pub struct RenderOptions<'g, N: Node<N>> {
  /// The viewport to render the node in.
  pub(crate) viewport: Viewport,
  /// The global context.
  pub(crate) global: &'g GlobalContext,
  /// The node to render.
  pub(crate) node: N,
  /// Whether to draw debug borders.
  #[builder(default)]
  pub(crate) draw_debug_border: bool,
}

/// Renders a node to an image.
pub fn render<'g, N: Node<N>>(options: RenderOptions<'g, N>) -> Result<RgbaImage, crate::Error> {
  let mut taffy = TaffyTree::new();

  let (tx, rx) = channel();
  let canvas = Canvas::new(tx);

  let render_context = (&options).into();

  let tree = NodeTree::from_node(&render_context, options.node);

  let root_node_id = tree.insert_into_taffy(&mut taffy);

  let available_space = Size {
    width: AvailableSpace::Definite(render_context.viewport.width as f32),
    height: AvailableSpace::Definite(render_context.viewport.height as f32),
  };

  taffy
    .compute_layout_with_measure(
      root_node_id,
      available_space,
      |known_dimensions, available_space, _node_id, node_context, _style| {
        if let Size {
          width: Some(width),
          height: Some(height),
        } = known_dimensions
        {
          return Size { width, height };
        }

        node_context
          .unwrap()
          .measure(available_space, known_dimensions)
      },
    )
    .unwrap();

  #[cfg(target_arch = "wasm32")]
  let canvas = {
    render_node(
      &mut taffy,
      root_node_id,
      &canvas,
      Point::ZERO,
      Affine::identity(),
    );

    drop(canvas);

    create_blocking_canvas_loop(render_context.viewport, rx)
  };

  #[cfg(not(target_arch = "wasm32"))]
  let canvas = {
    let handler =
      std::thread::spawn(move || create_blocking_canvas_loop(render_context.viewport, rx));

    render_node(
      &mut taffy,
      root_node_id,
      &canvas,
      Point::ZERO,
      Affine::identity(),
    );

    drop(canvas);

    handler.join().unwrap()
  };

  Ok(canvas)
}

fn create_transform(style: &InheritedStyle, layout: &Layout, context: &RenderContext) -> Affine {
  let mut transform = Affine::identity();

  let transform_origin = style.transform_origin.0.unwrap_or_default();

  let center = Point {
    x: transform_origin
      .x
      .to_length_unit()
      .resolve_to_px(context, layout.size.width),
    y: transform_origin
      .y
      .to_length_unit()
      .resolve_to_px(context, layout.size.height),
  };

  // According to https://www.w3.org/TR/css-transforms-2/#ctm
  // the order is `translate` -> `rotate` -> `scale` -> `transform`.
  // But we need to invert the order because of matrix multiplication.
  if let Some(node_transform) = &*style.transform {
    transform = transform * node_transform.to_affine(context, layout, center);
  }

  if let Some(scale) = *style.scale {
    transform = transform * Affine::scale(scale.into(), center);
  }

  if let Some(rotate) = *style.rotate {
    transform = transform * Affine::rotation(rotate, center);
  }

  if let Some(translate) = *style.translate {
    transform = transform
      * Affine::translation(Size {
        width: translate.x.resolve_to_px(context, layout.size.width),
        height: translate.y.resolve_to_px(context, layout.size.height),
      });
  }

  transform
}

fn render_node<'g, Nodes: Node<Nodes>>(
  taffy: &mut TaffyTree<NodeTree<'g, Nodes>>,
  node_id: NodeId,
  canvas: &Canvas,
  offset: Point<f32>,
  mut transform: Affine,
) {
  let mut layout = *taffy.layout(node_id).unwrap();
  let node_context = taffy.get_node_context_mut(node_id).unwrap();

  if node_context.context.opacity == 0.0 {
    return;
  }

  layout.location.x += offset.x;
  layout.location.y += offset.y;

  transform =
    transform * create_transform(&node_context.context.style, &layout, &node_context.context);

  node_context.context.transform = transform;

  // Draw the block node itself first
  if let Some(node) = &node_context.node {
    node.draw_on_canvas(&node_context.context, canvas, layout);
  }

  if node_context.should_construct_inline_layout() {
    let (inline_layout, _, boxes) = node_context.create_inline_layout(layout.content_box_size());
    let font_style = node_context
      .context
      .style
      .to_sized_font_style(&node_context.context);

    // Draw the inline layout without a callback first
    let positioned_inline_boxes = draw_inline_layout(
      &node_context.context,
      canvas,
      layout,
      inline_layout,
      &font_style,
    );

    // Then handle the inline boxes directly by zipping the node refs with their positioned boxes
    for (node, inline_box) in boxes.iter().zip(positioned_inline_boxes.iter()) {
      let mut render_context = node_context.context.clone();

      render_context.transform = render_context.transform
        * Affine::translation(Size {
          width: inline_box.x,
          height: inline_box.y,
        });

      node.draw_on_canvas(
        &render_context,
        canvas,
        Layout {
          size: Size {
            width: inline_box.width,
            height: inline_box.height,
          },
          ..Default::default()
        },
      );
    }
  }

  if node_context.context.draw_debug_border {
    draw_debug_border(canvas, layout, node_context.context.transform);
  }

  for child_id in taffy.children(node_id).unwrap() {
    render_node(taffy, child_id, canvas, layout.location, transform);
  }
}
