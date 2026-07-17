// Re-export the Embedder trait and MockEmbedder from wm-vector-db
pub use wm_vector_db::{Embedder, MockEmbedder};

use crate::models::{EmbedError, EmbedVector};

/// A no-op embedder that always returns an error.
pub struct NoopEmbedder {
    dim: usize,
}

impl Default for NoopEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

impl NoopEmbedder {
    pub fn new() -> Self {
        Self { dim: 0 }
    }
}

impl Embedder for NoopEmbedder {
    fn embed(&self, _text: &str) -> Result<EmbedVector, EmbedError> {
        Err(EmbedError::ModelNotLoaded("no embedder configured".into()))
    }
    fn is_loaded(&self) -> bool {
        false
    }
    fn model_name(&self) -> &str {
        "none"
    }
    fn output_dim(&self) -> usize {
        self.dim
    }
}
