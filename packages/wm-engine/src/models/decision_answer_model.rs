use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnswerValue {
    Bool(bool),
    Label(String),
    Labels(Vec<String>),
}
