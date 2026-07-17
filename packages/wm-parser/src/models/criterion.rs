use serde::{Deserialize, Serialize};

/// Acceptance criteria as stored in frontmatter (flat, unchecked by default).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterionFm {
    pub text: String,
    #[serde(default)]
    pub checked: bool,
}
