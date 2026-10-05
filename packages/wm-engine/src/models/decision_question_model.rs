use serde::{Deserialize, Serialize};

use super::decision_qtype_model::QType;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Question {
    pub id: String,
    #[serde(rename = "type")]
    pub qtype: QType,
    pub instructions: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub multi: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub levels: Vec<String>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

impl Question {
    pub fn same_shape(&self, other: &Question) -> bool {
        self.id == other.id
            && self.qtype == other.qtype
            && self.multi == other.multi
            && self.options == other.options
            && self.levels == other.levels
    }
}
