use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: String,
    pub tool_name: String,
    pub action: String,
    pub duration_ms: i64,
    pub result: String,
    pub error_message: Option<String>,
    pub entity_refs: Vec<String>,
}
