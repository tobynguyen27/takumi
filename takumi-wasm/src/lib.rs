//! WebAssembly bindings for Takumi.

#![deny(clippy::unwrap_used, clippy::expect_used)]
#![deny(missing_docs)]
#![allow(
  clippy::module_name_repetitions,
  clippy::missing_errors_doc,
  clippy::missing_panics_doc,
  clippy::must_use_candidate
)]

use std::{collections::HashSet, fmt::Display, sync::Arc};

use base64::{Engine, prelude::BASE64_STANDARD};
use serde::Deserialize;
use serde_bytes::ByteBuf;
use serde_wasm_bindgen::{from_value, to_value};
use takumi::{
  GlobalContext,
  layout::{
    DEFAULT_DEVICE_PIXEL_RATIO, DEFAULT_FONT_SIZE, Viewport,
    node::{Node, NodeKind},
  },
  parley::{FontWeight, fontique::FontInfoOverride},
  rendering::{
    AnimationFrame, ImageOutputFormat, RenderOptionsBuilder, encode_animated_png,
    encode_animated_webp, measure_layout, render, write_image,
  },
  resources::{image::load_image_source_from_bytes, task::FetchTaskCollection},
};
use wasm_bindgen::prelude::*;
use xxhash_rust::xxh3::{Xxh3DefaultBuilder, xxh3_64};

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type AnyNode = { type: string; [key: string]: any };

export type ByteBuf = Uint8Array | ArrayBuffer | Buffer;

export type RenderOptions = {
  /**
   * The width of the image. If not provided, the width will be automatically calculated based on the content.
   */
  width?: number,
  /**
   * The height of the image. If not provided, the height will be automatically calculated based on the content.
   */
  height?: number,
  /**
   * The format of the image.
   * @default "png"
   */
  format?: "png" | "jpeg" | "webp" | "raw",
  /**
   * The quality of JPEG format (0-100).
   */
  quality?: number,
  /**
   * The resources fetched externally. You should collect the fetch tasks first using `extractResourceUrls` and then pass the resources here.
   */
  fetchedResources?: ImageSource[],
  /**
   * Whether to draw debug borders.
   */
  drawDebugBorder?: boolean,
  /**
   * Defines the ratio resolution of the image to the physical pixels.
   * @default 1.0
   */
  devicePixelRatio?: number,
};

export type RenderAnimationOptions = {
  width: number,
  height: number,
  format?: "webp" | "apng",
  drawDebugBorder?: boolean,
};

export type FontDetails = {
  name?: string,
  data: ByteBuf,
  weight?: number,
  style?: "normal" | "italic" | "oblique",
};

export type ImageSource = {
  src: string,
  data: ByteBuf,
};

export type Font = FontDetails | ByteBuf;

export type ConstructRendererOptions = {
  /**
   * The images that needs to be preloaded into the renderer.
   */
  persistentImages?: ImageSource[],
  /**
   * The fonts being used.
   */
  fonts?: Font[],
};

export type MeasuredTextRun = {
  text: string,
  x: number,
  y: number,
  width: number,
  height: number,
};

export type MeasuredNode = {
  width: number,
  height: number,
  transform: [number, number, number, number, number, number],
  children: MeasuredNode[],
  runs: MeasuredTextRun[],
};

export type AnimationFrameSource = {
  node: AnyNode,
  durationMs: number,
};
"#;

#[wasm_bindgen]
extern "C" {
  /// JavaScript object representing a layout node.
  #[wasm_bindgen(typescript_type = "AnyNode")]
  #[derive(Debug)]
  pub type AnyNode;

  /// JavaScript object representing render options.
  #[wasm_bindgen(typescript_type = "RenderOptions")]
  pub type RenderOptionsType;

  /// JavaScript object representing animation render options.
  #[wasm_bindgen(typescript_type = "RenderAnimationOptions")]
  pub type RenderAnimationOptionsType;

  /// JavaScript object representing font details.
  #[wasm_bindgen(typescript_type = "FontDetails")]
  pub type FontDetailsType;

  /// JavaScript type for font input (FontDetails or ByteBuf).
  #[wasm_bindgen(typescript_type = "Font")]
  pub type FontType;

  /// JavaScript object representing renderer construction options.
  #[wasm_bindgen(typescript_type = "ConstructRendererOptions")]
  pub type ConstructRendererOptionsType;

  /// JavaScript object representing an image source.
  #[wasm_bindgen(typescript_type = "ImageSource")]
  pub type ImageSourceType;

  /// JavaScript object representing a measured node tree.
  #[wasm_bindgen(typescript_type = "MeasuredNode")]
  pub type MeasuredNodeType;

  /// JavaScript object representing an animation frame source.
  #[wasm_bindgen(typescript_type = "AnimationFrameSource")]
  pub type AnimationFrameSourceType;
}

/// Options for rendering an image.
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RenderOptions {
  /// The width of the image in pixels.
  width: Option<u32>,
  /// The height of the image in pixels.
  height: Option<u32>,
  /// The output image format (PNG, JPEG, or WebP).
  format: Option<OutputFormat>,
  /// The JPEG quality (0-100), if applicable.
  quality: Option<u8>,
  /// Pre-fetched image resources to use during rendering.
  fetched_resources: Option<Vec<ImageSource>>,
  /// Whether to draw debug borders around layout elements.
  draw_debug_border: Option<bool>,
  /// The device pixel ratio for scaling.
  device_pixel_ratio: Option<f32>,
}

/// Options for rendering an animated image.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RenderAnimationOptions {
  /// The width of each frame in pixels.
  width: u32,
  /// The height of each frame in pixels.
  height: u32,
  /// The output animation format (WebP or APNG).
  format: Option<AnimationOutputFormat>,
  /// Whether to draw debug borders around layout elements.
  draw_debug_border: Option<bool>,
}

/// Details for loading a custom font.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FontDetails {
  /// The name of the font family.
  name: Option<String>,
  /// The raw font data bytes.
  data: ByteBuf,
  /// The font weight (e.g., 400 for normal, 700 for bold).
  weight: Option<f64>,
  /// The font style (normal, italic, or oblique).
  style: Option<FontStyle>,
}

/// Font input, either as detailed object or raw buffer.
#[derive(Deserialize)]
#[serde(untagged)]
enum Font {
  /// Font loaded with detailed configuration.
  Object(FontDetails),
  /// Raw font buffer.
  Buffer(ByteBuf),
}

/// Options for constructing a Renderer instance.
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ConstructRendererOptions {
  /// The images that needs to be preloaded into the renderer.
  persistent_images: Option<Vec<ImageSource>>,
  /// The fonts being used.
  fonts: Option<Vec<Font>>,
}

/// An image source with its URL and raw data.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImageSource {
  /// The source URL of the image.
  src: Arc<str>,
  /// The raw image data bytes.
  data: ByteBuf,
}

/// Output format for static images.
#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum OutputFormat {
  /// PNG format.
  Png,
  /// JPEG format.
  Jpeg,
  /// WebP format.
  WebP,
  /// Raw pixels format.
  Raw,
}

impl From<OutputFormat> for ImageOutputFormat {
  fn from(format: OutputFormat) -> Self {
    match format {
      OutputFormat::Png => ImageOutputFormat::Png,
      OutputFormat::Jpeg => ImageOutputFormat::Jpeg,
      OutputFormat::WebP => ImageOutputFormat::WebP,
      OutputFormat::Raw => unreachable!("Raw format should be handled separately"),
    }
  }
}

/// Output format for animated images.
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum AnimationOutputFormat {
  /// Animated PNG format.
  APng,
  /// Animated WebP format.
  WebP,
}

/// Font style variants.
#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
enum FontStyle {
  /// Normal font style.
  Normal,
  /// Italic font style.
  Italic,
  /// Oblique font style.
  Oblique,
}

/// A single frame in an animation sequence.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnimationFrameSource {
  /// The node tree to render for this frame.
  node: NodeKind,
  /// The duration of this frame in milliseconds.
  duration_ms: u32,
}

impl From<FontStyle> for takumi::parley::FontStyle {
  fn from(style: FontStyle) -> Self {
    match style {
      FontStyle::Italic => takumi::parley::FontStyle::Italic,
      FontStyle::Oblique => takumi::parley::FontStyle::Oblique(None),
      FontStyle::Normal => takumi::parley::FontStyle::Normal,
    }
  }
}

fn map_error<E: Display>(err: E) -> js_sys::Error {
  js_sys::Error::new(&err.to_string())
}

type JsResult<T> = Result<T, js_sys::Error>;

#[derive(PartialEq, Eq, Hash)]
struct ImageCacheKey {
  src: Box<str>,
  data_hash: u64,
}

/// The main renderer for Takumi image rendering engine.
#[wasm_bindgen]
#[derive(Default)]
pub struct Renderer {
  context: GlobalContext,
  persistent_image_cache: HashSet<ImageCacheKey, Xxh3DefaultBuilder>,
}

#[wasm_bindgen]
impl Renderer {
  /// Creates a new Renderer instance.
  #[wasm_bindgen(constructor)]
  pub fn new(options: Option<ConstructRendererOptionsType>) -> JsResult<Renderer> {
    let options: ConstructRendererOptions = options
      .map(|options| from_value(options.into()).map_err(map_error))
      .transpose()?
      .unwrap_or_default();

    let mut renderer = Self::default();

    if let Some(fonts) = options.fonts {
      for font in fonts {
        renderer.load_font_internal(&font)?;
      }
    }

    if let Some(images) = options.persistent_images {
      for image in images {
        renderer.put_persistent_image_internal(&image)?;
      }
    }

    Ok(renderer)
  }

  /// @deprecated use `loadFont` instead.
  #[wasm_bindgen(js_name = loadFontWithInfo)]
  pub fn load_font_with_info(&mut self, font: FontType) -> JsResult<()> {
    self.load_font(font)
  }

  /// Loads a font into the renderer (internal version without JS conversion).
  fn load_font_internal(&mut self, font: &Font) -> JsResult<()> {
    match font {
      Font::Buffer(buffer) => {
        self
          .context
          .font_context
          .load_and_store(buffer, None, None)
          .map_err(map_error)?;
      }
      Font::Object(details) => {
        self
          .context
          .font_context
          .load_and_store(
            &details.data,
            Some(FontInfoOverride {
              family_name: details.name.as_deref(),
              style: details.style.map(Into::into),
              weight: details.weight.map(|weight| FontWeight::new(weight as f32)),
              axes: None,
              width: None,
            }),
            None,
          )
          .map_err(map_error)?;
      }
    }
    Ok(())
  }

  /// Loads a font into the renderer.
  #[wasm_bindgen(js_name = loadFont)]
  pub fn load_font(&mut self, font: FontType) -> JsResult<()> {
    let input: Font = from_value(font.into()).map_err(map_error)?;
    self.load_font_internal(&input)
  }

  /// Puts a persistent image into the renderer's internal store (internal version without JS conversion).
  fn put_persistent_image_internal(&mut self, data: &ImageSource) -> JsResult<()> {
    let key = ImageCacheKey {
      src: data.src.as_ref().into(),
      data_hash: xxh3_64(&data.data),
    };

    if self.persistent_image_cache.contains(&key) {
      return Ok(());
    }

    self.persistent_image_cache.insert(key);

    let image = load_image_source_from_bytes(&data.data).map_err(map_error)?;
    self
      .context
      .persistent_image_store
      .insert(data.src.to_string(), image);

    Ok(())
  }

  /// Puts a persistent image into the renderer's internal store.
  #[wasm_bindgen(js_name = putPersistentImage)]
  pub fn put_persistent_image(&mut self, data: ImageSourceType) -> JsResult<()> {
    let data: ImageSource = from_value(data.into()).map_err(map_error)?;
    self.put_persistent_image_internal(&data)
  }

  /// Clears the renderer's internal image store.
  #[wasm_bindgen(js_name = clearImageStore)]
  pub fn clear_image_store(&self) {
    self.context.persistent_image_store.clear();
  }

  /// Renders a node tree into an image buffer.
  #[wasm_bindgen]
  pub fn render(
    &self,
    node: AnyNode,
    options: Option<RenderOptionsType>,
  ) -> Result<Vec<u8>, JsValue> {
    let node: NodeKind = from_value(node.into()).map_err(map_error)?;
    let options: RenderOptions = options
      .map(|options| from_value(options.into()).map_err(map_error))
      .transpose()?
      .unwrap_or_default();

    self.render_internal(node, options)
  }

  fn render_internal(&self, node: NodeKind, options: RenderOptions) -> Result<Vec<u8>, JsValue> {
    let fetched_resources = options
      .fetched_resources
      .map(|resources| -> Result<_, JsValue> {
        resources
          .into_iter()
          .map(|source| {
            let image = load_image_source_from_bytes(&source.data).map_err(map_error)?;
            Ok((source.src, image))
          })
          .collect::<Result<_, JsValue>>()
      })
      .transpose()?
      .unwrap_or_default();

    let render_options = RenderOptionsBuilder::default()
      .viewport(Viewport {
        width: options.width,
        height: options.height,
        font_size: DEFAULT_FONT_SIZE,
        device_pixel_ratio: options
          .device_pixel_ratio
          .unwrap_or(DEFAULT_DEVICE_PIXEL_RATIO),
      })
      .draw_debug_border(options.draw_debug_border.unwrap_or_default())
      .fetched_resources(fetched_resources)
      .node(node)
      .global(&self.context)
      .build()
      .map_err(|e| JsValue::from_str(&format!("Failed to build render options: {e}")))?;

    let image = render(render_options).map_err(map_error)?;

    let format = options.format.unwrap_or(OutputFormat::Png);

    if format == OutputFormat::Raw {
      return Ok(image.into_raw());
    }

    let mut buffer = Vec::new();

    write_image(&image, &mut buffer, format.into(), options.quality).map_err(map_error)?;

    Ok(buffer)
  }

  /// Measures a node tree and returns layout information.
  #[wasm_bindgen(js_name = measure)]
  pub fn measure(
    &self,
    node: AnyNode,
    options: Option<RenderOptionsType>,
  ) -> Result<MeasuredNodeType, JsValue> {
    let node: NodeKind = from_value(node.into()).map_err(map_error)?;
    let options: RenderOptions = options
      .map(|options| from_value(options.into()).map_err(map_error))
      .transpose()?
      .unwrap_or_default();

    let fetched_resources = options
      .fetched_resources
      .map(|resources| -> Result<_, JsValue> {
        resources
          .into_iter()
          .map(|source| {
            let image = load_image_source_from_bytes(&source.data).map_err(map_error)?;
            Ok((source.src, image))
          })
          .collect::<Result<_, JsValue>>()
      })
      .transpose()?
      .unwrap_or_default();

    let render_options = RenderOptionsBuilder::default()
      .viewport(Viewport {
        width: options.width,
        height: options.height,
        font_size: DEFAULT_FONT_SIZE,
        device_pixel_ratio: options
          .device_pixel_ratio
          .unwrap_or(DEFAULT_DEVICE_PIXEL_RATIO),
      })
      .draw_debug_border(options.draw_debug_border.unwrap_or_default())
      .fetched_resources(fetched_resources)
      .node(node)
      .global(&self.context)
      .build()
      .map_err(|e| JsValue::from_str(&format!("Failed to build render options: {e}")))?;

    let layout = measure_layout(render_options).map_err(map_error)?;

    Ok(to_value(&layout).map_err(map_error)?.into())
  }

  /// Renders a node tree into a data URL.
  ///
  /// `raw` format is not supported for data URL.
  #[wasm_bindgen(js_name = "renderAsDataUrl")]
  pub fn render_as_data_url(&self, node: AnyNode, options: RenderOptionsType) -> JsResult<String> {
    let node: NodeKind = from_value(node.into()).map_err(map_error)?;
    let options: RenderOptions = from_value(options.into()).map_err(map_error)?;

    let format = options.format.unwrap_or(OutputFormat::Png);

    if format == OutputFormat::Raw {
      return Err(js_sys::Error::new(
        "Raw format is not supported for data URL",
      ));
    }

    let buffer = self.render_internal(node, options)?;

    let mut data_uri = String::new();

    data_uri.push_str("data:");
    data_uri.push_str(ImageOutputFormat::from(format).content_type());
    data_uri.push_str(";base64,");
    data_uri.push_str(&BASE64_STANDARD.encode(buffer));

    Ok(data_uri)
  }

  /// Renders an animation sequence into a buffer.
  #[wasm_bindgen(js_name = renderAnimation)]
  pub fn render_animation(
    &self,
    frames: Vec<AnimationFrameSourceType>,
    options: RenderAnimationOptionsType,
  ) -> Result<Vec<u8>, JsValue> {
    let frames: Vec<AnimationFrameSource> = from_value(frames.into()).map_err(map_error)?;
    let options: RenderAnimationOptions = from_value(options.into()).map_err(map_error)?;

    let rendered_frames: Vec<AnimationFrame> = frames
      .into_iter()
      .map(|frame| -> Result<AnimationFrame, JsValue> {
        let render_options = RenderOptionsBuilder::default()
          .viewport((options.width, options.height).into())
          .node(frame.node)
          .global(&self.context)
          .draw_debug_border(options.draw_debug_border.unwrap_or_default())
          .build()
          .map_err(|e| JsValue::from_str(&format!("Failed to build render options: {e}")))?;

        let image = render(render_options).map_err(map_error)?;
        Ok(AnimationFrame::new(image, frame.duration_ms))
      })
      .collect::<Result<Vec<_>, JsValue>>()?;

    let mut buffer = Vec::new();

    match options.format.unwrap_or(AnimationOutputFormat::WebP) {
      AnimationOutputFormat::WebP => {
        encode_animated_webp(&rendered_frames, &mut buffer, true, false, None)
          .map_err(map_error)?;
      }
      AnimationOutputFormat::APng => {
        encode_animated_png(&rendered_frames, &mut buffer, None).map_err(map_error)?;
      }
    }

    Ok(buffer)
  }
}

/// Collects the fetch task urls from the node.
#[wasm_bindgen(js_name = extractResourceUrls)]
pub fn extract_resource_urls(node: AnyNode) -> JsResult<Vec<String>> {
  let node: NodeKind = from_value(node.into()).map_err(map_error)?;

  let mut collection = FetchTaskCollection::default();

  node.collect_fetch_tasks(&mut collection);
  node.collect_style_fetch_tasks(&mut collection);

  Ok(
    collection
      .into_inner()
      .iter()
      .map(|task| task.to_string())
      .collect(),
  )
}

/// Collects the fetch task urls from the node.
/// @deprecated Use `extractResourceUrls` instead.
#[wasm_bindgen(js_name = collectNodeFetchTasks)]
pub fn collect_node_fetch_tasks(node: AnyNode) -> JsResult<Vec<String>> {
  extract_resource_urls(node)
}
