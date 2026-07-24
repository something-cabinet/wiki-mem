use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::state_model::SourceState;

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceEntry {
    pub id: String,
    pub original_path: Option<String>,
    pub stored_path: PathBuf,
    pub content_hash: String,
    pub state: SourceState,
    pub added_at: String,
    pub last_processed_at: Option<String>,
    pub page_refs: Vec<String>,
    pub page_count: usize,
    pub error_message: Option<String>,
    pub retry_count: usize,
}
