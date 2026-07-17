use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatternData {
    pub when_to_use: String,
    pub example: String,
}
