use std::collections::HashSet;

use napi::bindgen_prelude::*;
use takumi::resources::image::{PersistentImageStore, load_image_source_from_bytes};
use xxhash_rust::xxh3::{Xxh3DefaultBuilder, xxh3_64};

use crate::{map_error, renderer::ImageCacheKey};

pub struct PutPersistentImageTask<'s> {
  pub src: Option<String>,
  pub store: &'s PersistentImageStore,
  pub buffer: Buffer,
  pub(crate) persistent_image_cache: &'s mut HashSet<ImageCacheKey, Xxh3DefaultBuilder>,
}

impl Task for PutPersistentImageTask<'_> {
  type Output = ();
  type JsValue = ();

  fn compute(&mut self) -> Result<Self::Output> {
    let Some(src) = self.src.take() else {
      unreachable!()
    };

    let cache_key = ImageCacheKey {
      src: src.as_str().into(),
      data_hash: xxh3_64(&self.buffer),
    };

    if self.persistent_image_cache.contains(&cache_key) {
      return Ok(());
    }

    self.persistent_image_cache.insert(cache_key);

    let image = load_image_source_from_bytes(&self.buffer).map_err(map_error)?;
    self.store.insert(src, image);

    Ok(())
  }

  fn resolve(&mut self, _env: napi::Env, _output: Self::Output) -> napi::Result<Self::JsValue> {
    Ok(())
  }
}
