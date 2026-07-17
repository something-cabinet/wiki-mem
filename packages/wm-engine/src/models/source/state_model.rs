use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SourceState {
    Pending,
    Processing,
    Done,
    Error,
    Stale,
}
