use serde_json::{json, Value};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolError {
    pub code: &'static str,
    pub message: String,
    pub hint: Option<String>,
}

impl ToolError {
    pub fn required_field(field: impl Into<String>) -> Self {
        let f: String = field.into();
        Self { code: "REQUIRED_FIELD", message: format!("{} is required", f), hint: None }
    }

    pub fn not_found(entity: &str, detail: &str) -> Self {
        Self {
            code: "NOT_FOUND",
            message: format!("{} not found: {}", entity, detail),
            hint: Some(format!("Use the list tool to find available {}", entity)),
        }
    }

    pub fn no_project() -> Self {
        Self {
            code: "NO_PROJECT",
            message: "No project set. Call project.set first.".into(),
            hint: Some("Use project.detect to find projects, or project.set to select one.".into()),
        }
    }

    pub fn invalid_action(valid: &[&str]) -> Self {
        Self {
            code: "INVALID_ACTION",
            message: format!("Unknown action. Valid actions: {}", valid.join(", ")),
            hint: Some("Use help tool for detailed documentation.".into()),
        }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self { code: "INTERNAL_ERROR", message: msg.into(), hint: None }
    }

    pub fn to_json(&self) -> Value {
        let mut obj = json!({
            "error": {
                "code": self.code,
                "message": self.message,
            }
        });
        if let Some(ref hint) = self.hint {
            obj["error"]["hint"] = json!(hint);
        }
        obj
    }
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for ToolError {}

pub type ToolResult<T> = Result<T, ToolError>;
