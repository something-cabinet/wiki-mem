use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecisionData {
    pub context: String,
    pub options: Vec<String>,
    pub rationale: String,
    pub outcome: String,
    pub consequences: Option<String>,
}
