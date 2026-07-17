use serde::{Deserialize, Serialize};

/// General goal entry in frontmatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalEntry {
    pub description: String,
}
