use serde::{Deserialize, Serialize};
use super::doc_version_model::DocVersion;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocVersionHistory {
    pub entity_id: String,
    pub current_version: u32,
    pub versions: Vec<DocVersion>,
}
