use image::RgbaImage;
use wide::{u32x4, u32x8, u32x16};

use crate::rendering::{premultiply_alpha, unpremultiply_alpha};

const PIXEL_STRIDE: usize = 4;

/// Specifies the type of blur operation, which affects how the CSS radius is interpreted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlurType {
  /// CSS `filter: blur()` - radius equals σ (standard deviation).
  Filter,
  /// CSS `box-shadow` / `text-shadow` blur - radius equals 2σ.
  Shadow,
}

impl BlurType {
  #[inline]
  pub fn to_sigma(self, css_radius: f32) -> f32 {
    match self {
      BlurType::Filter => css_radius,
      BlurType::Shadow => css_radius * 0.5,
    }
  }

  #[inline]
  pub fn extent_multiplier(self) -> f32 {
    match self {
      BlurType::Filter => 3.0,
      BlurType::Shadow => 1.5,
    }
  }
}

/// Trait for SIMD vectors that can process pixel data.
/// This allows the same blur algorithm to work with different SIMD widths.
trait PixelVector:
  Copy
  + Default
  + std::ops::Add<Output = Self>
  + std::ops::Sub<Output = Self>
  + std::ops::Mul<Output = Self>
  + std::ops::Shr<i32, Output = Self>
{
  /// Number of u32 lanes (4, 8, or 16)
  const LANES: usize;

  /// Number of complete RGBA pixels per vector (1, 2, or 4)
  const PIXELS_PER_BATCH: usize = Self::LANES / 4;

  /// Create with all lanes set to the same value
  fn splat(value: u32) -> Self;

  /// Zero vector
  fn zero() -> Self;

  /// Load channels from buffer (loads LANES u32 values as u8->u32)
  fn load_channels(buffer: &[u8], offset: usize) -> Self;

  /// Store channels to buffer (stores LANES u32 values as u32->u8 clamped)
  fn store_channels(self, buffer: &mut [u8], offset: usize);
}

#[derive(Clone, Copy)]
struct BlurPassParams {
  width: u32,
  height: u32,
  radius: u32,
  stride: usize,
  mul_val: u32,
  shg: i32,
}

impl PixelVector for u32x4 {
  const LANES: usize = 4;

  #[inline(always)]
  fn splat(value: u32) -> Self {
    u32x4::new([value; 4])
  }

  #[inline(always)]
  fn zero() -> Self {
    u32x4::ZERO
  }

  #[inline(always)]
  fn load_channels(buffer: &[u8], offset: usize) -> Self {
    u32x4::new([
      buffer[offset] as u32,
      buffer[offset + 1] as u32,
      buffer[offset + 2] as u32,
      buffer[offset + 3] as u32,
    ])
  }

  #[inline(always)]
  fn store_channels(self, buffer: &mut [u8], offset: usize) {
    let arr: [u32; 4] = self.into();
    buffer[offset] = arr[0].min(255) as u8;
    buffer[offset + 1] = arr[1].min(255) as u8;
    buffer[offset + 2] = arr[2].min(255) as u8;
    buffer[offset + 3] = arr[3].min(255) as u8;
  }
}

impl PixelVector for u32x8 {
  const LANES: usize = 8;

  #[inline(always)]
  fn splat(value: u32) -> Self {
    u32x8::new([value; 8])
  }

  #[inline(always)]
  fn zero() -> Self {
    u32x8::ZERO
  }

  #[inline(always)]
  fn load_channels(buffer: &[u8], offset: usize) -> Self {
    u32x8::new([
      buffer[offset] as u32,
      buffer[offset + 1] as u32,
      buffer[offset + 2] as u32,
      buffer[offset + 3] as u32,
      buffer[offset + 4] as u32,
      buffer[offset + 5] as u32,
      buffer[offset + 6] as u32,
      buffer[offset + 7] as u32,
    ])
  }

  #[inline(always)]
  fn store_channels(self, buffer: &mut [u8], offset: usize) {
    let arr: [u32; 8] = self.into();
    for i in 0..8 {
      buffer[offset + i] = arr[i].min(255) as u8;
    }
  }
}

impl PixelVector for u32x16 {
  const LANES: usize = 16;

  #[inline(always)]
  fn splat(value: u32) -> Self {
    u32x16::new([value; 16])
  }

  #[inline(always)]
  fn zero() -> Self {
    u32x16::ZERO
  }

  #[inline(always)]
  fn load_channels(buffer: &[u8], offset: usize) -> Self {
    u32x16::new([
      buffer[offset] as u32,
      buffer[offset + 1] as u32,
      buffer[offset + 2] as u32,
      buffer[offset + 3] as u32,
      buffer[offset + 4] as u32,
      buffer[offset + 5] as u32,
      buffer[offset + 6] as u32,
      buffer[offset + 7] as u32,
      buffer[offset + 8] as u32,
      buffer[offset + 9] as u32,
      buffer[offset + 10] as u32,
      buffer[offset + 11] as u32,
      buffer[offset + 12] as u32,
      buffer[offset + 13] as u32,
      buffer[offset + 14] as u32,
      buffer[offset + 15] as u32,
    ])
  }

  #[inline(always)]
  fn store_channels(self, buffer: &mut [u8], offset: usize) {
    let arr: [u32; 16] = self.into();
    for i in 0..16 {
      buffer[offset + i] = arr[i].min(255) as u8;
    }
  }
}

#[derive(Clone, Copy)]
enum SimdBestFit {
  U32x4, // Baseline SIMD - 128-bit
  #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
  U32x8, // AVX2 - 256-bit
  #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
  U32x16, // AVX-512 - 512-bit
}

// Only for x86/x86_64 to extend above 128-bit SIMD
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn detect_best_simd() -> SimdBestFit {
  static BEST_SIMD: std::sync::OnceLock<SimdBestFit> = std::sync::OnceLock::new();

  let best_fit = BEST_SIMD.get_or_init(|| {
    if is_x86_feature_detected!("avx512f") {
      return SimdBestFit::U32x16;
    }
    if is_x86_feature_detected!("avx2") {
      return SimdBestFit::U32x8;
    }

    SimdBestFit::U32x4
  });

  *best_fit
}

// For non-x86/x86_64 there is no runtime dispatch needed
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
const fn detect_best_simd() -> SimdBestFit {
  SimdBestFit::U32x4
}

/// Applies a Gaussian approximation using 3-pass Box Blur with SIMD.
/// Automatically selects the best SIMD implementation available at runtime.
pub(crate) fn apply_blur(image: &mut RgbaImage, radius: f32, blur_type: BlurType) {
  let sigma = blur_type.to_sigma(radius);
  if sigma <= 0.5 {
    return;
  }

  let (width, height) = image.dimensions();
  if width == 0 || height == 0 {
    return;
  }

  // Detect best SIMD implementation once and cache it
  let simd_impl = detect_best_simd();

  match simd_impl {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    SimdBestFit::U32x16 => unsafe {
      apply_blur_avx512(image, sigma);
    },
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    SimdBestFit::U32x8 => unsafe {
      apply_blur_avx2(image, sigma);
    },
    SimdBestFit::U32x4 => apply_blur_impl::<u32x4>(image, sigma),
  }
}

/// AVX-512 implementation wrapper with target feature enabled
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx512f")]
unsafe fn apply_blur_avx512(image: &mut RgbaImage, sigma: f32) {
  apply_blur_impl::<u32x16>(image, sigma);
}

/// AVX2 implementation wrapper with target feature enabled
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn apply_blur_avx2(image: &mut RgbaImage, sigma: f32) {
  apply_blur_impl::<u32x8>(image, sigma);
}

#[inline(always)]
fn apply_blur_impl<V: PixelVector>(image: &mut RgbaImage, sigma: f32) {
  let box_radius = (((4.0 * sigma * sigma + 1.0).sqrt() - 1.0) * 0.5)
    .round()
    .max(1.0) as u32;

  let (width, height) = image.dimensions();

  for pixel in image.pixels_mut() {
    premultiply_alpha(pixel);
  }

  let mut temp_data = vec![0u8; (width * height * 4) as usize];
  let img_data = &mut **image;
  let stride = width as usize * PIXEL_STRIDE;
  let div = 2 * box_radius + 1;
  let (mul_val, shg) = compute_mul_shg(div);
  let pass_params = BlurPassParams {
    width,
    height,
    radius: box_radius,
    stride,
    mul_val,
    shg,
  };

  // 3-pass Box Blur to approximate Gaussian
  for _ in 0..3 {
    box_blur_h(img_data, &mut temp_data, pass_params);
    box_blur_v::<V>(&temp_data, img_data, pass_params);
  }

  for pixel in image.pixels_mut() {
    unpremultiply_alpha(pixel);
  }
}

/// Horizontal Box Blur Pass - Use u32x4 to process RGBA of one pixel with correct sliding window
fn box_blur_h(src: &[u8], dst: &mut [u8], params: BlurPassParams) {
  let r = params.radius as i32;
  let w = params.width as i32;
  let mul = <u32x4 as PixelVector>::splat(params.mul_val);
  let first_repeat = <u32x4 as PixelVector>::splat(r as u32 + 1);

  for y in 0..params.height {
    let line_offset = y as usize * params.stride;
    let mut sum = u32x4::ZERO;

    // Initialize sum with first pixel repeated (r+1) times
    let p_first = <u32x4 as PixelVector>::load_channels(src, line_offset);
    sum += p_first * first_repeat;

    // Add trailing edge
    for dx in 1..=r {
      let px = dx.min(w - 1);
      sum += <u32x4 as PixelVector>::load_channels(src, line_offset + (px as usize * PIXEL_STRIDE));
    }

    // Slide window across the whole row
    for x in 0..w {
      let out_offset = line_offset + (x as usize * PIXEL_STRIDE);
      <u32x4 as PixelVector>::store_channels((sum * mul) >> params.shg, dst, out_offset);

      let p_leaving_x = (x - r).max(0);
      let p_entering_x = (x + r + 1).min(w - 1);

      let p_leaving = <u32x4 as PixelVector>::load_channels(
        src,
        line_offset + (p_leaving_x as usize * PIXEL_STRIDE),
      );
      let p_entering = <u32x4 as PixelVector>::load_channels(
        src,
        line_offset + (p_entering_x as usize * PIXEL_STRIDE),
      );

      sum = sum + p_entering - p_leaving;
    }
  }
}

/// Vertical Box Blur Pass - Correctly leverages wide SIMD by processing multiple columns in parallel
fn box_blur_v<V: PixelVector>(src: &[u8], dst: &mut [u8], params: BlurPassParams) {
  let r = params.radius as i32;
  let h = params.height as i32;
  let mul = V::splat(params.mul_val);
  let first_repeat = V::splat(r as u32 + 1);

  let batch_size = V::PIXELS_PER_BATCH as u32;

  let mut x = 0;
  while x + batch_size <= params.width {
    let col_offset = x as usize * PIXEL_STRIDE;
    let mut sum = V::zero();

    // Initialize sum with first pixel of each column repeated (r+1) times
    let p_first = V::load_channels(src, col_offset);
    sum = sum + p_first * first_repeat;

    // Add trailing edge
    for dy in 1..=r {
      let py = dy.min(h - 1);
      sum = sum + V::load_channels(src, col_offset + (py as usize * params.stride));
    }

    // Slide window down the columns
    for y in 0..h {
      let out_offset = col_offset + (y as usize * params.stride);
      V::store_channels((sum * mul) >> params.shg, dst, out_offset);

      let p_leaving_y = (y - r).max(0);
      let p_entering_y = (y + r + 1).min(h - 1);

      let p_leaving = V::load_channels(src, col_offset + (p_leaving_y as usize * params.stride));
      let p_entering = V::load_channels(src, col_offset + (p_entering_y as usize * params.stride));

      sum = sum + p_entering - p_leaving;
    }

    x += batch_size;
  }

  // Handle remaining columns with u32x4
  let mul_scalar = <u32x4 as PixelVector>::splat(params.mul_val);
  let first_repeat_scalar = <u32x4 as PixelVector>::splat(r as u32 + 1);
  for x in x..params.width {
    let col_offset = x as usize * PIXEL_STRIDE;
    let mut sum = u32x4::ZERO;

    let p_first = <u32x4 as PixelVector>::load_channels(src, col_offset);
    sum += p_first * first_repeat_scalar;

    for dy in 1..=r {
      let py = dy.min(h - 1);
      sum += <u32x4 as PixelVector>::load_channels(src, col_offset + (py as usize * params.stride));
    }

    for y in 0..h {
      let out_offset = col_offset + (y as usize * params.stride);
      <u32x4 as PixelVector>::store_channels((sum * mul_scalar) >> params.shg, dst, out_offset);

      let p_leaving_y = (y - r).max(0);
      let p_entering_y = (y + r + 1).min(h - 1);

      let p_leaving = <u32x4 as PixelVector>::load_channels(
        src,
        col_offset + (p_leaving_y as usize * params.stride),
      );
      let p_entering = <u32x4 as PixelVector>::load_channels(
        src,
        col_offset + (p_entering_y as usize * params.stride),
      );

      sum = sum + p_entering - p_leaving;
    }
  }
}

#[inline(always)]
fn compute_mul_shg(d: u32) -> (u32, i32) {
  let shg = 23;
  let mul = ((1u64 << shg) as f64 / d as f64).round() as u32;
  (mul, shg)
}
