use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DecisionFm {
    pub context: String,
    #[serde(default)]
    pub options: Vec<String>,
    pub rationale: String,
    pub outcome: String,
    #[serde(default)]
    pub consequences: Option<String>,
}
