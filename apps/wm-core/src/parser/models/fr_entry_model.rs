use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FrEntry {
    pub id: String,
    pub description: String,
}
