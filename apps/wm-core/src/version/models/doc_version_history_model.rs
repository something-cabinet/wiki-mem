use super::doc_version_model::DocVersion;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DocVersionHistory {
    pub entity_id: String,
    pub current_version: u32,
    pub versions: Vec<DocVersion>,
}
