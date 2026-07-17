use serde::{Deserialize, Serialize};
use crate::status::MemoryStatus;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub status: Option<MemoryStatus>,
}
