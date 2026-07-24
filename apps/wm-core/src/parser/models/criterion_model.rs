use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterionFm {
    pub text: String,
    #[serde(default)]
    pub checked: bool,
}
