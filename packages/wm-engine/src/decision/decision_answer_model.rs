use std::collections::BTreeMap;

use crate::models::AnswerValue;

#[derive(Clone, Debug, PartialEq)]
pub struct DecisionAnswer {
    pub value: AnswerValue,
    pub probability: f64,
    pub distribution: Option<BTreeMap<String, f64>>,
}
