use crate::mcp::prelude::*;



fn read_log_lines() -> Vec<String> {
    let log_path = std::path::Path::new(".wm").join("wm_log.jsonl");
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
struct WmLogSinceInput {
    #[schemars(description = "Marker string to search from")]
    marker: String,
    #[allow(dead_code)] // populated by serde, reserved for future use
    #[schemars(description = "Max entries")]
    limit: Option<i32>,
}

#[derive(Deserialize, JsonSchema)]
struct WmLogFilterInput {
    #[schemars(description = "Text to search for")]
    text: String,
    #[allow(dead_code)] // populated by serde, reserved for future use
    #[schemars(description = "Max entries")]
    limit: Option<i32>,
}

pub fn register(registry: &mut ToolRegistry, _engine: Arc<EngineState>) {
    registry.register_typed(
        "wm_log.recent",
        "Recent log entries",
        move |input: WmLogRecentInput| {
            let count = input.limit.unwrap_or(20) as usize;
            let all_lines = read_log_lines();
            let total = all_lines.len();
            let start = total.saturating_sub(count);
            let lines: Vec<&str> = all_lines[start..].iter().map(String::as_str).collect();
            Ok(serde_json::json!({
                "entries": lines,
                "total": total,
            }))
        },
    );

    registry.register_typed(
        "wm_log.since",
        "Log entries since a marker",
        move |input: WmLogSinceInput| {
            let marker = input.marker;
            let lines: Vec<String> = read_log_lines()
                .into_iter()
                .skip_while(|line| !line.contains(&marker))
                .skip(1)
                .collect();
            Ok(serde_json::json!({
                "entries": lines,
                "total": lines.len(),
            }))
        },
    );

    registry.register_typed(
        "wm_log.filter",
        "Filter log entries by text",
        move |input: WmLogFilterInput| {
            let text = input.text;
            let lines: Vec<String> = read_log_lines()
                .into_iter()
                .filter(|line| line.to_lowercase().contains(&text.to_lowercase()))
                .collect();
            Ok(serde_json::json!({
                "entries": lines,
                "total": lines.len(),
            }))
        },
    );
}
