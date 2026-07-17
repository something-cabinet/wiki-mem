use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub text: String,
    pub checked: bool,
}
