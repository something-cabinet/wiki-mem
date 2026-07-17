use serde::{Deserialize, Serialize};
use super::field_change::FieldChange;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocVersion {
    pub id: String,
    pub version: u32,
    pub timestamp: String,
    pub author: Option<String>,
    pub changes: Vec<FieldChange>,
    pub path: String,
    pub compacted: bool,
}
