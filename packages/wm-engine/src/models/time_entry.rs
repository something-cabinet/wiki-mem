use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeEntry {
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_s: Option<u64>,
    pub note: Option<String>,
}
