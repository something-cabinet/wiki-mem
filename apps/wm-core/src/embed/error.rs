use std::fmt;

#[derive(Debug, Clone)]
pub enum EmbedError {
    ModelNotLoaded(String),
    Inference(String),
    Tokenization(String),
    DimensionMismatch { expected: usize, actual: usize },
    BatchTooLarge { size: usize, max: usize },
    ModelNotFound(String),
    Download(String),
}

impl fmt::Display for EmbedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmbedError::ModelNotLoaded(msg) => write!(f, "model not loaded: {}", msg),
            EmbedError::Inference(msg) => write!(f, "inference error: {}", msg),
            EmbedError::Tokenization(msg) => write!(f, "tokenization error: {}", msg),
            EmbedError::DimensionMismatch { expected, actual } => {
                write!(f, "dimension mismatch: expected {}, got {}", expected, actual)
            }
            EmbedError::BatchTooLarge { size, max } => {
                write!(f, "batch size {} exceeds limit {}", size, max)
            }
            EmbedError::ModelNotFound(msg) => write!(f, "model file not found: {}", msg),
            EmbedError::Download(msg) => write!(f, "download error: {}", msg),
        }
    }
}

impl std::error::Error for EmbedError {}
