use std::sync::Arc;

/// A task for resolving a resource URL.
pub struct FetchTask {
  /// The URL to resolve.
  pub url: Arc<str>,
}

impl FetchTask {
  /// Create a new [`FetchTask`] for the given URL.
  pub fn new(url: Arc<str>) -> Self {
    Self { url }
  }
}
