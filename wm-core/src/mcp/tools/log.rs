use std::sync::Arc;

use crate::engine::EngineState;
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;

/// Register log tool handlers
pub fn register(registry: &mut ToolRegistry, _engine: Arc<EngineState>) {
    registry.register_with_desc(
        "log.recent",
        "Recent log entries",
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

    registry.register_with_desc(
        "log.since",
        "Log entries since a marker",
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

    registry.register_with_desc(
        "log.filter",
        "Filter log entries by text",
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
