use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrEntry {
    pub id: String,
    pub description: String,
}
