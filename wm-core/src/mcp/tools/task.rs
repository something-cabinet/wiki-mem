use std::sync::Arc;

use crate::engine::{EngineState, PageType};
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;
use crate::page;

/// Register task tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_with_desc(
        "task.check_ac",
        "Check an acceptance criterion",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let ac_indices = args.optional_string_array("criteria");
            let indices: Vec<u64> = ac_indices.iter().filter_map(|s| s.parse().ok()).collect();
            let update = serde_json::json!({ "checked_ac": indices });
            page::update_page(&e, &id, &update)?;
            Ok(serde_json::json!({ "id": id, "checked": indices }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "task.uncheck_ac",
        "Uncheck an acceptance criterion",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let ac_indices = args.optional_string_array("criteria");
            let indices: Vec<u64> = ac_indices.iter().filter_map(|s| s.parse().ok()).collect();
            let update = serde_json::json!({ "unchecked_ac": indices });
            page::update_page(&e, &id, &update)?;
            Ok(serde_json::json!({ "id": id, "unchecked": indices }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc("task.board", "Task board grouped by status", Arc::new(move |_params| {
        let board = crate::task::task_board(&e);
        Ok(serde_json::json!(board))
    }));

    let e = engine.clone();
    registry.register_with_desc(
        "task.list",
        "List tasks with optional filters (status, label, limit)",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let filter_status = args.optional_string("status").map(|s| s.to_lowercase());
            let filter_label = args.optional_string("label").map(|s| s.to_lowercase());
            let limit = args.optional_int("limit").unwrap_or(50);

            let snapshot = e.graph.load();
            let graph = &snapshot.0;

            let mut tasks: Vec<serde_json::Value> = Vec::new();

            for idx in graph.node_indices() {
                let meta = &graph[idx];
                if meta.page_type != PageType::Task {
                    continue;
                }

                // Filter by status
                if let Some(ref status) = filter_status {
                    let task_status = format!("{:?}", meta.status).to_lowercase();
                    if task_status != *status {
                        continue;
                    }
                }

                // Filter by label (match against tags)
                if let Some(ref label) = filter_label {
                    let has_label = meta.tags.iter().any(|t| t.to_lowercase() == *label);
                    if !has_label {
                        continue;
                    }
                }

                // Get description from file content
                let description = extract_task_description(&meta.path);

                tasks.push(serde_json::json!({
                    "id": meta.id,
                    "title": meta.title,
                    "status": format!("{:?}", meta.status).to_lowercase(),
                    "priority": meta.priority.as_ref().map(|p| format!("{:?}", p).to_lowercase()),
                    "labels": meta.tags,
                    "description": description,
                }));

                if tasks.len() >= limit {
                    break;
                }
            }

            Ok(serde_json::json!({
                "tasks": tasks,
                "total": tasks.len(),
            }))
        }),
    );
}

/// Extract a truncated description from a task page file
fn extract_task_description(path: &std::path::Path) -> String {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };

    // Skip frontmatter
    let body = if let Some(end) = content.find("\n---") {
        let after = &content[end + 4..];
        // Also handle `---` at end of frontmatter on same line
        if after.starts_with('\n') {
            &after[1..]
        } else {
            after
        }
    } else {
        &content
    };

    let body = body.trim();

    // Take first non-empty line as description, truncated
    let first_line = body.lines().find(|l| !l.trim().is_empty());
    match first_line {
        Some(line) => {
            let line = line.trim();
            if line.len() > 200 {
                format!("{}...", &line[..197])
            } else {
                line.to_string()
            }
        }
        None => String::new(),
    }
}
