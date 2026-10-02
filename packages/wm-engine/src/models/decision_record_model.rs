use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::decision_answer_model::AnswerValue;
use super::decision_question_model::Question;

pub const RECORD_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRecord {
    pub schema_version: u32,
    pub state: String,
    pub questions: Vec<Question>,
    #[serde(default)]
    pub answers: BTreeMap<String, AnswerValue>,
}
