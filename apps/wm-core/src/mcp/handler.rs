use wm_error::{ToolError, ToolResult};
use serde_json::Value;

/// Typed argument extraction from JSON-RPC params
pub struct ToolArgs(Value);

impl ToolArgs {
    pub fn new(params: Value) -> Self {
        Self(params)
    }

    pub fn require_string(&self, key: &str) -> ToolResult<String> {
        match self.0.get(key).and_then(|v| v.as_str()) {
            Some(s) if !s.is_empty() => Ok(s.to_string()),
            _ => Err(ToolError::required_field(key)),
        }
    }

    pub fn optional_string(&self, key: &str) -> Option<String> {
        self.0
            .get(key)
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from)
    }

    pub fn optional_text(&self, key: &str) -> Option<String> {
        self.optional_string(key)
            .map(|s| wm_util::unescape_text(&s))
    }

    pub fn optional_int(&self, key: &str) -> Option<usize> {
        self.0.get(key).and_then(|v| v.as_u64()).map(|n| n as usize)
    }

    pub fn optional_bool(&self, key: &str) -> bool {
        self.0.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
    }

    pub fn optional_string_array(&self, key: &str) -> Vec<String> {
        self.0
            .get(key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }
}
