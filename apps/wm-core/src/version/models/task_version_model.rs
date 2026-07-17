use serde::{Deserialize, Serialize};
use crate::version::models::field_change_model::FieldChange;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskVersion {
    pub id: String,
    pub version: u32,
    pub timestamp: String,
    pub author: Option<String>,
    pub changes: Vec<FieldChange>,
    pub compacted: bool,
}
