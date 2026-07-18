use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::engine::EngineState;
use crate::mcp::transport::ToolRegistry;

use crate::page;

// ─── Action enum ────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
enum WmTimeAction {
    #[schemars(description = "Start time tracking on a task")]
    Start { id: String },
    #[schemars(description = "Stop time tracking, record elapsed")]
    Stop { id: String, #[allow(dead_code)] note: Option<String> },
    #[schemars(description = "Manually add time to a task")]
    Add { id: String, duration: String, #[allow(dead_code)] note: Option<String> },
    #[schemars(description = "Time report across all tasks")]
    Report { #[allow(dead_code)] group_by: Option<String> },
}

// ─── Output types ───────────────────────────────────────────

#[derive(Serialize)]
struct WmTimeStartOutput {
    id: String,
    time_started: String,
    status: String,
}

#[derive(Serialize)]
struct WmTimeStopOutput {
    id: String,
    time_spent: String,
    status: String,
}

#[derive(Serialize)]
struct WmTimeAddOutput {
    id: String,
    time_spent: String,
    status: String,
}

#[derive(Serialize)]
struct WmTimeReportOutput {
    tasks: Vec<serde_json::Value>,
    total_tasks: usize,
    total_hours: f64,
    total_estimated_hours: f64,
}

// ─── Helpers ────────────────────────────────────────────────

/// Parse a duration string like "2h 30m" or "45m" into total minutes.
fn parse_duration_to_minutes(s: &str) -> f64 {
    let s = s.trim();
    if s.is_empty() { return 0.0; }
    let mut minutes = 0.0;
    if let Some(h) = s.split('h').next().and_then(|p| p.trim().parse::<f64>().ok()) {
        minutes += h * 60.0;
    } else if s.contains('h') {
        tracing::warn!("Failed to parse duration: {}", s);
    }
    if let Some(m_part) = s.rsplit('h').next() {
        let trimmed = m_part.trim().trim_end_matches('m');
        if let Ok(m) = trimmed.parse::<f64>() {
            minutes += m;
        } else if !trimmed.is_empty() {
            tracing::warn!("Failed to parse duration: {}", s);
        }
    }
    minutes
}

// ─── Tool Registration ──────────────────────────────────────

/// Register the single wm_time tool
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    registry.register_typed(
        "wm_time",
        "Time tracking operations: start, stop, add, report",
        move |input: WmTimeAction| -> Result<serde_json::Value, wm_error::ToolError> {
            match input {
                // ── Start ──────────────────────────────────────
                WmTimeAction::Start { id } => {
                    let now = chrono::Utc::now().to_rfc3339();
                    let params = page::PageUpdateParams {
                        time_started: Some(now.clone()),
                        ..Default::default()
                    };
                    page::update_page(&engine, &id, &params)?;
                    Ok(serde_json::to_value(WmTimeStartOutput {
                        id,
                        time_started: now,
                        status: "started".to_string(),
                    }).unwrap_or(serde_json::Value::Null))
                }

                // ── Stop ───────────────────────────────────────
                WmTimeAction::Stop { id, note: _ } => {
                    let snapshot = engine.graph.load();
                    let index = &snapshot.1;
                    let node_idx = index
                        .get(&id)
                        .ok_or_else(|| wm_error::ToolError::not_found("page", &id))?;
                    let meta = &snapshot.0[*node_idx];
                    let file_path = &meta.path;

                    let content = std::fs::read_to_string(file_path)
                        .map_err(|e| wm_error::ToolError::internal(format!("read error: {}", e)))?;
                    let (fm, _) = crate::parser::extract_frontmatter(&content);

                    let time_started = fm
                        .as_ref()
                        .and_then(|f| f.time_started.as_deref())
                        .unwrap_or("");

                    let now = chrono::Utc::now();
                    let elapsed_minutes = if let Ok(started) = chrono::DateTime::parse_from_rfc3339(time_started) {
                        let dur = now.signed_duration_since(started);
                        (dur.num_hours() * 60 + dur.num_minutes() % 60) as f64
                    } else {
                        0.0
                    };

                    let existing_spent = fm.as_ref().and_then(|f| f.time_spent.as_deref()).unwrap_or("");
                    let existing_minutes = parse_duration_to_minutes(existing_spent);
                    let total_minutes = existing_minutes + elapsed_minutes;
                    let total_hours = (total_minutes / 60.0).floor() as i64;
                    let total_mins = (total_minutes % 60.0) as i64;
                    let total = format!("{}h {}m", total_hours, total_mins);

                    let params = page::PageUpdateParams {
                        time_spent: Some(total.clone()),
                        ..Default::default()
                    };
                    page::update_page(&engine, &id, &params)?;
                    Ok(serde_json::to_value(WmTimeStopOutput {
                        id,
                        time_spent: total,
                        status: "stopped".to_string(),
                    }).unwrap_or(serde_json::Value::Null))
                }

                // ── Add ────────────────────────────────────────
                WmTimeAction::Add { id, duration, note: _ } => {
                    let snapshot = engine.graph.load();
                    let index = &snapshot.1;
                    let node_idx = index
                        .get(&id)
                        .ok_or_else(|| wm_error::ToolError::not_found("page", &id))?;
                    let meta = &snapshot.0[*node_idx];
                    let file_path = &meta.path;

                    let content = std::fs::read_to_string(file_path)
                        .map_err(|e| wm_error::ToolError::internal(format!("read error: {}", e)))?;
                    let (fm, _) = crate::parser::extract_frontmatter(&content);

                    let existing_spent = fm.as_ref().and_then(|f| f.time_spent.as_deref()).unwrap_or("");
                    let existing_minutes = parse_duration_to_minutes(existing_spent);
                    let added_minutes = parse_duration_to_minutes(&duration);
                    let total_minutes = existing_minutes + added_minutes;
                    let total_hours = (total_minutes / 60.0).floor() as i64;
                    let total_mins = (total_minutes % 60.0) as i64;
                    let total = format!("{}h {}m", total_hours, total_mins);

                    let params = page::PageUpdateParams {
                        time_spent: Some(total.clone()),
                        ..Default::default()
                    };
                    page::update_page(&engine, &id, &params)?;
                    Ok(serde_json::to_value(WmTimeAddOutput {
                        id,
                        time_spent: total,
                        status: "added".to_string(),
                    }).unwrap_or(serde_json::Value::Null))
                }

                // ── Report ─────────────────────────────────────
                WmTimeAction::Report { group_by: _ } => {
                    let snapshot = engine.graph.load();
                    let graph = &snapshot.0;
                    let mut tasks: Vec<serde_json::Value> = Vec::new();
                    let mut total_hours = 0f64;
                    let mut total_estimated_hours = 0f64;

                    for idx in graph.node_indices() {
                        let meta = &graph[idx];
                        if meta.page_type != crate::engine::PageType::Task {
                            continue;
                        }

                        let file_path = &meta.path;
                        if !file_path.exists() {
                            continue;
                        }
                        let content = std::fs::read_to_string(file_path).unwrap_or_default();
                        let (fm, _) = crate::parser::extract_frontmatter(&content);

                        let time_spent = fm
                            .as_ref()
                            .and_then(|f| f.time_spent.as_deref())
                            .unwrap_or("");
                        let time_started = fm
                            .as_ref()
                            .and_then(|f| f.time_started.as_deref())
                            .unwrap_or("");
                        let estimate = fm.as_ref().and_then(|f| f.estimate);

                        if let Some(h) = time_spent
                            .split('h')
                            .next()
                            .and_then(|s| s.trim().parse::<f64>().ok())
                        {
                            total_hours += h;
                        }

                        if let Some(est) = estimate {
                            total_estimated_hours += est as f64;
                        }

                        if !time_spent.is_empty() || !time_started.is_empty() || estimate.is_some() {
                            tasks.push(serde_json::json!({
                                "id": meta.id,
                                "title": meta.title,
                                "time_spent": time_spent,
                                "time_started": time_started,
                                "estimate": estimate,
                            }));
                        }
                    }

                    let total_tasks = tasks.len();
                    Ok(serde_json::to_value(WmTimeReportOutput {
                        tasks,
                        total_tasks,
                        total_hours,
                        total_estimated_hours,
                    }).unwrap_or(serde_json::Value::Null))
                }
            }
        },
    );
}
