use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct AcceptanceCriterionFm {
    pub text: String,
    #[serde(default)]
    pub checked: bool,
}
