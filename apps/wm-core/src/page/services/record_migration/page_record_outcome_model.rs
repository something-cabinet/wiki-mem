use serde::Serialize;

use super::skip_reason_model::SkipReason;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "outcome")]
pub enum PageRecordOutcome {
    WouldConvert {
        #[serde(skip_serializing)]
        body: String,
    },
    AlreadyRecord,
    Skip { reason: SkipReason },
}
