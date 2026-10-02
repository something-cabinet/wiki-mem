use serde::Serialize;

use super::page_record_outcome_model::PageRecordOutcome;

/// The planner's decision for one parsed page file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PageRecordPlan {
    /// Wiki-relative path (e.g. `decisions/choose-db.md`).
    pub path: String,
    /// Record-bearing type name resolved from the page's directory.
    pub page_type: String,
    /// Conversion decision.
    pub outcome: PageRecordOutcome,
}
