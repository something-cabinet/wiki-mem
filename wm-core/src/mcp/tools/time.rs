use std::sync::Arc;

use crate::engine::EngineState;
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;
use crate::page;

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

/// Register time tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_with_desc(
        "wm_time.start",
        "Start time tracking on a task",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let now = chrono::Utc::now().to_rfc3339();
            let update = serde_json::json!({ "time_started": now });
            page::update_page(&e, &id, &update)?;
            Ok(serde_json::json!({ "id": id, "time_started": now, "status": "started" }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_time.stop",
        "Stop time tracking, record elapsed",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;

            let snapshot = e.graph.load();
            let index = &snapshot.1;
            let node_idx = index
                .get(&id)
                .ok_or_else(|| crate::error::ToolError::not_found("page", &id))?;
            let meta = &snapshot.0[*node_idx];
            let file_path = &meta.path;

            let content = std::fs::read_to_string(file_path)
                .map_err(|e| crate::error::ToolError::internal(format!("read error: {}", e)))?;
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

            let update = serde_json::json!({
                "time_spent": total,
                "time_started": serde_json::Value::Null,
            });
            page::update_page(&e, &id, &update)?;
            Ok(serde_json::json!({ "id": id, "time_spent": total, "status": "stopped" }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_time.add",
        "Manually add time to a task",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let duration = args.require_string("duration")?;

            let snapshot = e.graph.load();
            let index = &snapshot.1;
            let node_idx = index
                .get(&id)
                .ok_or_else(|| crate::error::ToolError::not_found("page", &id))?;
            let meta = &snapshot.0[*node_idx];
            let file_path = &meta.path;

            let content = std::fs::read_to_string(file_path)
                .map_err(|e| crate::error::ToolError::internal(format!("read error: {}", e)))?;
            let (fm, _) = crate::parser::extract_frontmatter(&content);

            let existing_spent = fm.as_ref().and_then(|f| f.time_spent.as_deref()).unwrap_or("");
            let existing_minutes = parse_duration_to_minutes(existing_spent);
            let added_minutes = parse_duration_to_minutes(&duration);
            let total_minutes = existing_minutes + added_minutes;
            let total_hours = (total_minutes / 60.0).floor() as i64;
            let total_mins = (total_minutes % 60.0) as i64;
            let total = format!("{}h {}m", total_hours, total_mins);

            let update = serde_json::json!({ "time_spent": total });
            page::update_page(&e, &id, &update)?;
            Ok(serde_json::json!({ "id": id, "time_spent": total, "status": "added" }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_time.report",
        "Time report across all tasks",
        Arc::new(move |_params| {
            let snapshot = e.graph.load();
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

            Ok(serde_json::json!({
                "tasks": tasks,
                "total_tasks": tasks.len(),
                "total_hours": total_hours,
                "total_estimated_hours": total_estimated_hours,
            }))
        }),
    );
}
