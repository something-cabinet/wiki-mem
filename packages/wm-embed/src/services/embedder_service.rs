// Re-export the Embedder trait and MockEmbedder from vector_db
pub use crate::vector_db::{Embedder, MockEmbedder};

use crate::vector_db::{EmbedError, EmbedVector};

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
        Err(EmbedError::SemanticUnavailable(
            "no embedder configured — ONNX model not loaded".into(),
        ))
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
