use serde::{Deserialize, Serialize};

/// Decision record stored in frontmatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionFm {
    pub context: String,
    #[serde(default)]
    pub options: Vec<String>,
    pub rationale: String,
    pub outcome: String,
    #[serde(default)]
    pub consequences: Option<String>,
}
