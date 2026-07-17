use serde::{Deserialize, Serialize};

/// Functional requirement entry in frontmatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrEntry {
    pub id: String,
    pub description: String,
}
