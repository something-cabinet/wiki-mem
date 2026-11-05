use serde::Serialize;

use super::page_record_outcome_model::PageRecordOutcome;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PageRecordPlan {
    pub path: String,
    pub page_type: String,
    pub outcome: PageRecordOutcome,
}
