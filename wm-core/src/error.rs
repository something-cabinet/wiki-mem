use serde_json::{json, Value};
use std::error::Error as StdError;
use std::fmt;

/// Structured error for MCP tool handlers.
/// Carries a machine-readable `code`, human-readable `message`,
/// optional `hint`, and optional chained `source` error.
#[derive(Debug)]
pub struct ToolError {
    pub code: &'static str,
    pub message: String,
    pub hint: Option<String>,
    pub source: Option<Box<dyn StdError + Send + Sync>>,
}

impl ToolError {
    pub fn required_field(field: impl Into<String>) -> Self {
        let f: String = field.into();
        Self {
            code: "REQUIRED_FIELD",
            message: format!("{} is required", f),
            hint: None,
            source: None,
        }
    }

    pub fn not_found(entity: &str, detail: &str) -> Self {
        Self {
            code: "NOT_FOUND",
            message: format!("{} not found: {}", entity, detail),
            hint: Some(format!("Use the list tool to find available {}", entity)),
            source: None,
        }
    }

    pub fn no_project() -> Self {
        Self {
            code: "NO_PROJECT",
            message: "No project set. Call project.set first.".into(),
            hint: Some("Use project.detect to find projects, or project.set to select one.".into()),
            source: None,
        }
    }

    pub fn invalid_action(valid: &[&str]) -> Self {
        Self {
            code: "INVALID_ACTION",
            message: format!("Unknown action. Valid actions: {}", valid.join(", ")),
            hint: Some("Use help tool for detailed documentation.".into()),
            source: None,
        }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self {
            code: "INTERNAL_ERROR",
            message: msg.into(),
            hint: None,
            source: None,
        }
    }

    /// Create from an I/O error with operation context.
    pub fn io_error(op: impl Into<String>, path: impl Into<String>, err: std::io::Error) -> Self {
        Self {
            code: "IO_ERROR",
            message: format!("{} failed on {}: {}", op.into(), path.into(), err),
            hint: None,
            source: Some(Box::new(err)),
        }
    }

    /// Create from a serialization/deserialization error.
    pub fn serde_error(op: impl Into<String>, err: impl StdError + Send + Sync + 'static) -> Self {
        Self {
            code: "SERDE_ERROR",
            message: format!("{} failed: {}", op.into(), err),
            hint: None,
            source: Some(Box::new(err)),
        }
    }

    /// Create from a lock poison error.
    pub fn lock_poisoned(resource: impl Into<String>) -> Self {
        Self {
            code: "LOCK_POISONED",
            message: format!("{} lock poisoned", resource.into()),
            hint: Some("Restart the application to recover.".into()),
            source: None,
        }
    }

    pub fn to_json(&self) -> Value {
        let mut obj = json!({
            "code": self.code,
            "message": self.message,
        });
        if let Some(ref hint) = self.hint {
            obj["hint"] = json!(hint);
        }
        obj
    }
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(ref source) = self.source {
            write!(f, "\n  caused by: {}", source)?;
        }
        Ok(())
    }
}

impl std::error::Error for ToolError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|s| s.as_ref() as &(dyn StdError + 'static))
    }
}

pub type ToolResult<T> = Result<T, ToolError>;

// Implement From<T> for common error types
impl From<std::io::Error> for ToolError {
    fn from(err: std::io::Error) -> Self {
        Self {
            code: "IO_ERROR",
            message: err.to_string(),
            hint: None,
            source: Some(Box::new(err)),
        }
    }
}
