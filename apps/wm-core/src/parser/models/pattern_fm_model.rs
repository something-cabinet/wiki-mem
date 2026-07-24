use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternFm {
    pub when_to_use: String,
    pub example: String,
}
