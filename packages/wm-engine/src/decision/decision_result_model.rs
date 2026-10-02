use std::collections::BTreeMap;

use super::decision_answer_model::DecisionAnswer;

#[derive(Clone, Debug, PartialEq)]
pub struct DecisionResult {
    pub page: Option<String>,
    pub model: String,
    pub answers: BTreeMap<String, DecisionAnswer>,
    pub chunks: usize,
    pub truncated: bool,
}
