use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NfrEntry {
    pub id: String,
    pub description: String,
}
