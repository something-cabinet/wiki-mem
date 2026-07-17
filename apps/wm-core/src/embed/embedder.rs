use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::error::EmbedError;
use super::vector::EmbedVector;

pub trait Embedder: Send + Sync {
    fn embed(&self, text: &str) -> Result<EmbedVector, EmbedError>;
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<EmbedVector>, EmbedError> {
        texts.iter().map(|t| self.embed(t)).collect()
    }
    fn is_loaded(&self) -> bool;
    fn model_name(&self) -> &str;
    fn output_dim(&self) -> usize;
}

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

pub struct MockEmbedder {
    dim: usize,
}

impl MockEmbedder {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }

    fn hash_vec(&self, text: &str) -> EmbedVector {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let seed = hasher.finish();

        let mut rng = oorandom::Rand64::new(seed.into());
        let mut vec = Vec::with_capacity(self.dim);
        for _ in 0..self.dim {
            vec.push(rng.rand_float() as f32);
        }
        EmbedVector(vec).normalized()
    }
}

impl Embedder for MockEmbedder {
    fn embed(&self, text: &str) -> Result<EmbedVector, EmbedError> {
        Ok(self.hash_vec(text))
    }
    fn is_loaded(&self) -> bool {
        true
    }
    fn model_name(&self) -> &str {
        "mock"
    }
    fn output_dim(&self) -> usize {
        self.dim
    }
}
