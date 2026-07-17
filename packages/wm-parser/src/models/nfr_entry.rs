use serde::{Deserialize, Serialize};

/// Non-functional requirement entry in frontmatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NfrEntry {
    pub id: String,
    pub description: String,
}
