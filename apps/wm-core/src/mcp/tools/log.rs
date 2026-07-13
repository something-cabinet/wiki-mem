use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;

use crate::engine::EngineState;
use crate::mcp::transport::ToolRegistry;
use crate::mcp::typed::TypedRegister;

// ─── Input types ───────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
struct WmLogRecentInput {
    #[schemars(description = "Number of entries")]
    limit: Option<i32>,
}

#[derive(Deserialize, JsonSchema)]
struct WmLogSinceInput {
    #[schemars(description = "Marker string to search from")]
    marker: String,
    #[schemars(description = "Max entries")]
    limit: Option<i32>,
}

#[derive(Deserialize, JsonSchema)]
struct WmLogFilterInput {
    #[schemars(description = "Text to search for")]
    text: String,
    #[schemars(description = "Max entries")]
    limit: Option<i32>,
}

/// Register log tool handlers
pub fn register(registry: &mut ToolRegistry, _engine: Arc<EngineState>) {
    registry.register_read(
        "wm_log.recent",
        "Recent log entries",
        move |input: WmLogRecentInput| {
            let count = input.limit.unwrap_or(20) as usize;
            let log_path = std::path::Path::new(".wm").join("wm_log.jsonl");
            let content = std::fs::read_to_string(&log_path).unwrap_or_default();
            let all_lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
            let total = all_lines.len();
            let start = total.saturating_sub(count);
            let lines: Vec<&str> = all_lines[start..].to_vec();
            Ok(serde_json::json!({
                "entries": lines,
                "total": total,
            }))
        },
    );

    registry.register_read(
        "wm_log.since",
        "Log entries since a marker",
        move |input: WmLogSinceInput| {
            let marker = input.marker;
            let log_path = std::path::Path::new(".wm").join("wm_log.jsonl");
            let content = std::fs::read_to_string(&log_path).unwrap_or_default();
            let lines: Vec<&str> = content
                .lines()
                .filter(|l| !l.trim().is_empty())
                .skip_while(|line| !line.contains(&marker))
                .skip(1)
                .collect();
            Ok(serde_json::json!({
                "entries": lines,
                "total": lines.len(),
            }))
        },
    );

    registry.register_read(
        "wm_log.filter",
        "Filter log entries by text",
        move |input: WmLogFilterInput| {
            let text = input.text;
            let log_path = std::path::Path::new(".wm").join("wm_log.jsonl");
            let content = std::fs::read_to_string(&log_path).unwrap_or_default();
            let lines: Vec<&str> = content
                .lines()
                .filter(|l| !l.trim().is_empty())
                .filter(|line| line.to_lowercase().contains(&text.to_lowercase()))
                .collect();
            Ok(serde_json::json!({
                "entries": lines,
                "total": lines.len(),
            }))
        },
    );
}
