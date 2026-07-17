use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::status::PageStatus;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PageType {
    Task,
    Spec,
    Concept,
    Pattern,
    Decision,
    Memory,
    Howto,
    Reference,
    #[serde(rename = "note")]
    Note,
}

impl PageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PageType::Task => "task",
            PageType::Spec => "spec",
            PageType::Concept => "concept",
            PageType::Pattern => "pattern",
            PageType::Decision => "decision",
            PageType::Memory => "memory",
            PageType::Howto => "howto",
            PageType::Reference => "reference",
            PageType::Note => "note",
        }
    }

    pub fn allowed_statuses(&self) -> &[PageStatus] {
        use PageStatus::*;
        match self {
            PageType::Task => &[Todo, InProgress, InReview, Done, Blocked, Cancelled],
            PageType::Spec => &[Draft, Reviewed, Approved, Superseded],
            PageType::Decision => &[Draft, Approved, Superseded, Rejected, Archived],
            _ => &[Draft, Reviewed, Approved, Archived],
        }
    }

    pub fn priority_rank(&self) -> u8 {
        match self {
            PageType::Task => 7,
            PageType::Spec => 6,
            PageType::Pattern => 5,
            PageType::Concept => 4,
            PageType::Decision => 3,
            PageType::Memory => 2,
            PageType::Howto => 2,
            PageType::Reference => 1,
            PageType::Note => 0,
        }
    }
}
