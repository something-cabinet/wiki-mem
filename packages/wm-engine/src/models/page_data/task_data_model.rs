use serde::{Deserialize, Serialize};

use super::acceptance_criterion_model::AcceptanceCriterion;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskData {
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub estimate: Option<u32>,
    pub prerequisites: Vec<String>,
    pub difficulty: Option<String>,
    pub time_spent: Option<String>,
    pub time_entries: Vec<crate::models::TimeEntry>,
    pub implementation_plan: Option<String>,
    pub implementation_notes: Option<String>,
}
