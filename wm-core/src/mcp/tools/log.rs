use std::sync::Arc;

use serde_json::json;

use crate::engine::EngineState;
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;

/// Register log tool handlers
pub fn register(registry: &mut ToolRegistry, _engine: Arc<EngineState>) {
    registry.register_with_schema(
        "wm_log.recent",
        "Recent log entries",
        json!({
            "type": "object",
            "properties": {
                "limit": { "type": "integer", "description": "Number of entries", "default": 20 }
            }
        }),
        Arc::new(|params| {
            let args = ToolArgs::new(params);
            let count = args.optional_int("count").unwrap_or(20);
            let log_path = std::path::Path::new(".wm").join("wiki").join("log.md");
            let content = std::fs::read_to_string(&log_path).unwrap_or_default();
            let all_lines: Vec<&str> = content.lines().collect();
            let total = all_lines.len();
            let start = total.saturating_sub(count);
            let lines: Vec<&str> = all_lines[start..].to_vec();
            Ok(serde_json::json!({
                "entries": lines,
                "total": total,
            }))
        }),
    );

    registry.register_with_schema(
        "wm_log.since",
        "Log entries since a marker",
        json!({
            "type": "object",
            "properties": {
                "marker": { "type": "string", "description": "Marker string to search from" },
                "limit": { "type": "integer", "description": "Max entries" }
            },
            "required": ["marker"]
        }),
        Arc::new(|params| {
            let args = ToolArgs::new(params);
            let marker = args.require_string("marker")?;
            let log_path = std::path::Path::new(".wm").join("wiki").join("log.md");
            let content = std::fs::read_to_string(&log_path).unwrap_or_default();
            let lines: Vec<&str> = content
                .lines()
                .skip_while(|line| !line.contains(&marker))
                .skip(1)
                .collect();
            Ok(serde_json::json!({
                "entries": lines,
                "total": lines.len(),
            }))
        }),
    );

    registry.register_with_schema(
        "wm_log.filter",
        "Filter log entries by text",
        json!({
            "type": "object",
            "properties": {
                "text": { "type": "string", "description": "Text to search for" },
                "limit": { "type": "integer", "description": "Max entries" }
            },
            "required": ["text"]
        }),
        Arc::new(|params| {
            let args = ToolArgs::new(params);
            let text = args.require_string("text")?;
            let log_path = std::path::Path::new(".wm").join("wiki").join("log.md");
            let content = std::fs::read_to_string(&log_path).unwrap_or_default();
            let lines: Vec<&str> = content
                .lines()
                .filter(|line| line.to_lowercase().contains(&text.to_lowercase()))
                .collect();
            Ok(serde_json::json!({
                "entries": lines,
                "total": lines.len(),
            }))
        }),
    );
}
