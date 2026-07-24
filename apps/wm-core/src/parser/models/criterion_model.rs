use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AcceptanceCriterionFm {
    pub text: String,
    #[serde(default)]
    pub checked: bool,
}
