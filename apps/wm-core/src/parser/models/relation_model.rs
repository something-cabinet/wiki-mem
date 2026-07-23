use serde::{Deserialize, Serialize};

/// A single relation entry (from `relates_to` list in frontmatter).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    #[serde(rename = "type")]
    pub edge_type: String,
    pub target: String,
}
