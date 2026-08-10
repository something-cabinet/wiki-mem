use crate::mcp::prelude::*;
use wm_constants::*;

/// Read the project's audit/log file. Resolved against the engine's project
/// root rather than the process CWD so `wm_log` finds `.wm/log.jsonl` no
/// matter where the daemon was launched from.
fn read_log_lines(engine: &EngineState) -> Vec<String> {
    let project_root = engine
        .project_root
        .read()
        .map(|r| r.clone())
        .unwrap_or_default();
    let log_path = project_root.join(WM_DIR).join(LOG_FILE);
    std::fs::read_to_string(&log_path)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(String::from)
        .collect()
}

#[derive(Deserialize, JsonSchema)]
struct WmLogRecentInput {
    #[schemars(description = "Number of entries")]
    limit: Option<i32>,
}

#[derive(Deserialize, JsonSchema)]
struct WmLogLimitSchema {
    #[serde(rename = "limit")]
    #[schemars(description = "Max entries")]
    _limit: Option<i32>,
}

#[derive(Deserialize, JsonSchema)]
struct WmLogSinceInput {
    #[schemars(description = "Marker string to search from")]
    marker: String,
    #[serde(flatten)]
    _schema: WmLogLimitSchema,
}

#[derive(Deserialize, JsonSchema)]
struct WmLogFilterInput {
    #[schemars(description = "Text to search for")]
    text: String,
    #[serde(flatten)]
    _schema: WmLogLimitSchema,
}

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    registry.register_typed(
        "wm_log.recent",
        "Recent log entries",
        {
            let engine = engine.clone();
            move |input: WmLogRecentInput| {
                let count = usize::try_from(input.limit.unwrap_or(20)).unwrap_or(20);
                let all_lines = read_log_lines(&engine);
                let total = all_lines.len();
                let start = total.saturating_sub(count);
                let lines: Vec<&str> = all_lines[start..].iter().map(String::as_str).collect();
                Ok(serde_json::json!({
                    "entries": lines,
                    "total": total,
                }))
            }
        },
    );

    registry.register_typed(
        "wm_log.since",
        "Log entries since a marker",
        {
            let engine = engine.clone();
            move |input: WmLogSinceInput| {
                let marker = input.marker;
                let lines: Vec<String> = read_log_lines(&engine)
                    .into_iter()
                    .skip_while(|line| !line.contains(&marker))
                    .skip(1)
                    .collect();
                Ok(serde_json::json!({
                    "entries": lines,
                    "total": lines.len(),
                }))
            }
        },
    );

    registry.register_typed(
        "wm_log.filter",
        "Filter log entries by text",
        {
            let engine = engine.clone();
            move |input: WmLogFilterInput| {
                let text = input.text;
                let lines: Vec<String> = read_log_lines(&engine)
                    .into_iter()
                    .filter(|line| line.to_lowercase().contains(&text.to_lowercase()))
                    .collect();
                Ok(serde_json::json!({
                    "entries": lines,
                    "total": lines.len(),
                }))
            }
        },
    );
}
