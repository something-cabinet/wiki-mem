use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NfrEntry {
    pub id: String,
    pub description: String,
}
