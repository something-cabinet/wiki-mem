use thiserror::Error;

#[derive(Debug, Error)]
pub enum LspError {
    #[error("Language server not found: {language}. {install_hint}")]
    Unavailable { language: String, install_hint: String },
    #[error("Language server is starting (indexing)")]
    Starting,
    #[error("Language server crashed: {language}")]
    Crashed { language: String },
    #[error("LSP request timed out: {operation}")]
    Timeout { operation: String },
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("JSON-RPC error: {0}")]
    Protocol(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
