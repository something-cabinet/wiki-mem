use serde::{Deserialize, Serialize};
use wm_constants::EMBED_BATCH_SIZE;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model_name: String,
    pub dimensions: u32,
    pub batch_size: u32,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model_name: "bge-small-en-v1.5".into(),
            dimensions: 384,
            batch_size: EMBED_BATCH_SIZE as u32,
        }
    }
}
