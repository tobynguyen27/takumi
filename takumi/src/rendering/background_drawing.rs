use std::{iter::successors, sync::Arc};

use image::{
  Rgba, RgbaImage,
  imageops::{FilterType, resize},
};
use taffy::{Layout, Point};

use crate::{
  layout::style::{
    BackgroundImage, BackgroundImages, BackgroundPosition, BackgroundPositions, BackgroundRepeat,
    BackgroundRepeatStyle, BackgroundRepeats, BackgroundSize, BackgroundSizes, Gradient,
    ImageScalingAlgorithm, LengthUnit, PositionComponent, PositionKeywordX, PositionKeywordY,
  },
  rendering::{BorderProperties, Canvas, RenderContext},
};

pub(crate) fn resolve_length_against_area(
  unit: LengthUnit,
  area: u32,
  context: &RenderContext,
) -> u32 {
  match unit {
    LengthUnit::Auto => area,
    _ => unit.resolve_to_px(context, area as f32).max(0.0) as u32,
  }
}

pub(crate) fn resolve_background_size(
  size: BackgroundSize,
  area: (u32, u32),
  context: &RenderContext,
) -> (u32, u32) {
  match size {
    BackgroundSize::Explicit { width, height } => (
      resolve_length_against_area(width, area.0, context),
      resolve_length_against_area(height, area.1, context),
    ),
    // as we only support gradients for now, we can just use the area size
    // if we want to support images, we need to resolve based on the image size
    _ => area,
  }
}

pub(crate) fn resolve_length_unit_to_position_component(
  length: LengthUnit,
  available: i32,
  context: &RenderContext,
) -> i32 {
  match length {
    LengthUnit::Auto => available / 2,
    _ => length.resolve_to_px(context, available as f32) as i32,
  }
}

pub(crate) fn resolve_position_component_x(
  comp: BackgroundPosition,
  tile_w: u32,
  area_w: u32,
  context: &RenderContext,
) -> i32 {
  let available = area_w.saturating_sub(tile_w) as i32;
  match comp.x {
    PositionComponent::KeywordX(PositionKeywordX::Left) => 0,
    PositionComponent::KeywordX(PositionKeywordX::Center) => available / 2,
    PositionComponent::KeywordX(PositionKeywordX::Right) => available,
    PositionComponent::KeywordY(_) => available / 2,
    PositionComponent::Length(length) => {
      resolve_length_unit_to_position_component(length, available, context)
    }
  }
}

pub(crate) fn resolve_position_component_y(
  comp: BackgroundPosition,
  tile_h: u32,
  area_h: u32,
  context: &RenderContext,
) -> i32 {
  let available = area_h.saturating_sub(tile_h) as i32;
  match comp.y {
    PositionComponent::KeywordY(PositionKeywordY::Top) => 0,
    PositionComponent::KeywordY(PositionKeywordY::Center) => available / 2,
    PositionComponent::KeywordY(PositionKeywordY::Bottom) => available,
    PositionComponent::KeywordX(_) => available / 2,
    PositionComponent::Length(length) => {
      resolve_length_unit_to_position_component(length, available, context)
    }
  }
}

/// Rasterize a single background image (gradient) into a tile of the given size.
/// resolving non-px stop units using the provided `RenderContext`.
pub(crate) fn render_gradient_tile(
  image: &BackgroundImage,
  tile_w: u32,
  tile_h: u32,
  context: &RenderContext,
) -> RgbaImage {
  match image {
    BackgroundImage::Linear(gradient) => gradient.to_image(tile_w, tile_h, context),
    BackgroundImage::Radial(gradient) => gradient.to_image(tile_w, tile_h, context),
    BackgroundImage::Noise(noise) => noise.to_image(tile_w, tile_h, context),
  }
}

/// Resolve tile image, positions along X and Y for a background-like layer.
/// Returns (tile_image, tile_w, tile_h, xs, ys).
pub(crate) fn resolve_layer_tiles(
  image: &BackgroundImage,
  pos: BackgroundPosition,
  size: BackgroundSize,
  repeat: BackgroundRepeat,
  area_w: u32,
  area_h: u32,
  context: &RenderContext,
) -> (RgbaImage, Vec<i32>, Vec<i32>) {
  // Compute tile size
  let (mut tile_w, mut tile_h) = resolve_background_size(size, (area_w, area_h), context);

  if tile_w == 0 || tile_h == 0 {
    return (
      RgbaImage::from_pixel(0, 0, Rgba([0, 0, 0, 0])),
      vec![],
      vec![],
    );
  }

  // Build tile image (use context-aware resolver where possible)
  let mut tile_image = render_gradient_tile(image, tile_w, tile_h, context);

  // Handle round adjustment (rescale per axis)
  let xs: Vec<i32> = match repeat.x {
    BackgroundRepeatStyle::Repeat => {
      let origin_x = resolve_position_component_x(pos, tile_w, area_w, context);
      collect_repeat_tile_positions(area_w, tile_w, origin_x)
    }
    BackgroundRepeatStyle::NoRepeat => {
      let origin_x = resolve_position_component_x(pos, tile_w, area_w, context);
      vec![origin_x]
    }
    BackgroundRepeatStyle::Space => collect_spaced_tile_positions(area_w, tile_w),
    BackgroundRepeatStyle::Round => {
      let (px, new_w) = collect_stretched_tile_positions(area_w, tile_w);
      if new_w != tile_w {
        tile_w = new_w;
        tile_image = resize(&tile_image, tile_w, tile_h, FilterType::CatmullRom);
      }
      px
    }
  };

  let ys: Vec<i32> = match repeat.y {
    BackgroundRepeatStyle::Repeat => {
      let origin_y = resolve_position_component_y(pos, tile_h, area_h, context);
      collect_repeat_tile_positions(area_h, tile_h, origin_y)
    }
    BackgroundRepeatStyle::NoRepeat => {
      let origin_y = resolve_position_component_y(pos, tile_h, area_h, context);
      vec![origin_y]
    }
    BackgroundRepeatStyle::Space => collect_spaced_tile_positions(area_h, tile_h),
    BackgroundRepeatStyle::Round => {
      let (py, new_h) = collect_stretched_tile_positions(area_h, tile_h);
      if new_h != tile_h {
        tile_h = new_h;
        tile_image = resize(&tile_image, tile_w, tile_h, FilterType::CatmullRom);
      }
      py
    }
  };

  (tile_image, xs, ys)
}

/// Collects a list of tile positions to place along an axis.
/// Starts from the "origin" and collects tile positions until the "area_size" is reached.
pub(crate) fn collect_repeat_tile_positions(
  area_size: u32,
  tile_size: u32,
  origin: i32,
) -> Vec<i32> {
  if tile_size == 0 {
    return Vec::new();
  }

  // Find first position, should be <= 0
  let mut start = origin;
  if start > 0 {
    let n = ((start as f32) / tile_size as f32).ceil() as i32;
    start -= n * tile_size as i32;
  }

  successors(Some(start), |&x| Some(x + tile_size as i32))
    .take_while(|&x| x < area_size as i32)
    .collect()
}

/// Collects evenly spaced tile positions along an axis for `background-repeat: space`.
/// Distributes gaps between tiles so the first and last touch the edges.
pub(crate) fn collect_spaced_tile_positions(area_size: u32, tile_size: u32) -> Vec<i32> {
  if tile_size == 0 {
    return vec![];
  }

  // Calculate number of tiles that fit in the area
  let count = area_size / tile_size;

  // Fast path: if there's only one tile, center it
  if count <= 1 {
    return vec![((area_size as i32 - tile_size as i32) / 2)];
  }

  // Calculate gap between tiles
  let gap = (area_size - count * tile_size) / (count - 1);
  let step = tile_size as i32 + gap as i32;

  successors(Some(0i32), move |&x| Some(x + step))
    .take(count as usize)
    .collect()
}

/// Collects stretched tile positions along an axis for `background-repeat: round`.
/// Rounds the size of the tile to fill the area.
/// Returns the positions and the new tile size.
pub(crate) fn collect_stretched_tile_positions(area_size: u32, tile_size: u32) -> (Vec<i32>, u32) {
  if tile_size == 0 || area_size == 0 {
    return (vec![], tile_size);
  }

  // Calculate number of tiles that fit in the area, at least 1
  let count = (area_size as f32 / tile_size as f32).max(1.0) as u32;

  let new_tile_size = (area_size as f32 / count as f32) as u32;

  let positions = successors(Some(0i32), move |&x| Some(x + new_tile_size as i32))
    .take(count as usize)
    .collect();

  (positions, new_tile_size)
}

pub(crate) fn resolve_layers_tiles(
  images: &BackgroundImages,
  positions: Option<&BackgroundPositions>,
  sizes: Option<&BackgroundSizes>,
  repeats: Option<&BackgroundRepeats>,
  context: &RenderContext,
  layout: Layout,
) -> Vec<(RgbaImage, Vec<i32>, Vec<i32>)> {
  let last_position = positions
    .and_then(|p| p.0.last().copied())
    .unwrap_or_default();
  let last_size = sizes.and_then(|s| s.0.last().copied()).unwrap_or_default();
  let last_repeat = repeats
    .and_then(|r| r.0.last().copied())
    .unwrap_or_default();

  let mut tiles = Vec::new();

  // Paint each background layer in order
  for (i, image) in images.0.iter().enumerate() {
    let pos = positions
      .and_then(|p| p.0.get(i).copied())
      .unwrap_or(last_position);
    let size = sizes.and_then(|s| s.0.get(i).copied()).unwrap_or(last_size);
    let repeat = repeats
      .and_then(|r| r.0.get(i).copied())
      .unwrap_or(last_repeat);

    let resolved = resolve_layer_tiles(
      image,
      pos,
      size,
      repeat,
      layout.size.width as u32,
      layout.size.height as u32,
      context,
    );

    if resolved.1.is_empty()
      || resolved.2.is_empty()
      || resolved.0.width() == 0
      || resolved.0.height() == 0
    {
      continue;
    }

    tiles.push(resolved);
  }

  tiles
}

/// Draw layered backgrounds (gradients) with support for background-size, -position, and -repeat.
pub(crate) fn draw_background_layers(
  tiles: Vec<(RgbaImage, Vec<i32>, Vec<i32>)>,
  radius: BorderProperties,
  context: &RenderContext,
  canvas: &Canvas,
  layout: Layout,
) {
  for (tile_image, xs, ys) in tiles {
    let tile_image = Arc::new(tile_image);

    for y in &ys {
      for x in &xs {
        // radius is Copy, pass by value
        canvas.overlay_image(
          tile_image.clone(),
          Point {
            x: *x + layout.location.x as i32,
            y: *y + layout.location.y as i32,
          },
          radius,
          context.transform,
          ImageScalingAlgorithm::Auto,
          context.style.filter.0.to_owned(),
        );
      }
    }
  }
}
