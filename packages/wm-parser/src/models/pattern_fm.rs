use serde::{Deserialize, Serialize};

/// Pattern entry in frontmatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternFm {
    /// Context/when to apply the pattern
    pub when_to_use: String,
    /// Concrete example of the pattern in use
    pub example: String,
}
