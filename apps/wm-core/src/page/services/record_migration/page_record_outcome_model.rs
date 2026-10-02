use serde::Serialize;

use super::skip_reason_model::SkipReason;

/// The planner's decision for a single record-bearing page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "outcome")]
pub enum PageRecordOutcome {
    /// The page would be rewritten to the canonical record body.
    WouldConvert {
        /// Fully rendered record body kept for `--apply`; excluded from reports.
        #[serde(skip_serializing)]
        body: String,
    },
    /// The body already starts with `schema_version: 1`; conversion is a no-op.
    AlreadyRecord,
    /// The page cannot be safely converted; it is left untouched.
    Skip { reason: SkipReason },
}
