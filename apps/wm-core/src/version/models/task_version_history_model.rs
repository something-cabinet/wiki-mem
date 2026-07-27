use super::task_version_model::TaskVersion;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskVersionHistory {
    pub entity_id: String,
    pub current_version: u32,
    pub versions: Vec<TaskVersion>,
}
