use std::sync::Arc;

use serde_json::json;

use crate::engine::{EngineState, PageType};
use crate::error::ToolError;
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;
use crate::page;

/// Register task tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    // ── wm_task.create ──────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_schema(
        "wm_task.create",
        "Create a task wiki page. Sets type: task automatically.",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Task page ID" },
                "title": { "type": "string", "description": "Task title" },
                "status": { "type": "string", "description": "Task status: todo/in_progress/done/blocked", "default": "todo" },
                "priority": { "type": "string", "description": "Priority: low/medium/high/urgent", "default": "medium" },
                "tags": { "type": "array", "items": { "type": "string" }, "description": "Tags" },
                "acceptance_criteria": { "type": "array", "items": { "type": "string" }, "description": "Acceptance criteria" },
                "assignee": { "type": "string", "description": "Assignee name" },
                "content": { "type": "string", "description": "Task description content" }
            },
            "required": ["id", "title"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let title = args.require_string("title")?;
            let status = args.optional_string("status").unwrap_or_else(|| "todo".to_string());
            let priority = args.optional_string("priority").unwrap_or_else(|| "medium".to_string());
            let tags = args.optional_string_array("tags");
            let acceptance_criteria = args.optional_string_array("acceptance_criteria");
            let assignee = args.optional_string("assignee");
            let content = args.optional_text("content").unwrap_or_default();

            // Build frontmatter YAML
            let mut frontmatter = format!(
                "title: {}\ntype: task\nstatus: {}\npriority: {}\n",
                title, status, priority
            );

            if !tags.is_empty() {
                frontmatter.push_str(&format!("tags: [{}]\n", tags.join(", ")));
            }

            if !acceptance_criteria.is_empty() {
                frontmatter.push_str("acceptance_criteria:\n");
                for ac in &acceptance_criteria {
                    frontmatter.push_str(&format!("  - {{text: \"{}\", checked: false}}\n", ac));
                }
            }

            if let Some(ref assignee) = assignee {
                frontmatter.push_str(&format!("assignee: {}\n", assignee));
            }

            let path = format!("tasks/{}", id);
            let page_id = page::create_page(&e, &path, &frontmatter, &content)?;

            Ok(json!({
                "id": page_id,
                "title": title,
                "status": status,
                "priority": priority,
                "tags": tags,
                "acceptance_criteria": acceptance_criteria,
                "assignee": assignee,
                "type": "task",
            }))
        }),
    );

    // ── wm_task.get ─────────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_schema(
        "wm_task.get",
        "Get a task by ID. Only returns pages with type: task.",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Task page ID" }
            },
            "required": ["id"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;

            // Look up in graph
            let snapshot = e.graph.load();
            let index = &snapshot.1;
            let node_idx = index
                .get(&id)
                .ok_or_else(|| ToolError::not_found("task", &id))?;
            let meta = &snapshot.0[*node_idx];

            // Only return for task pages
            if meta.page_type != PageType::Task {
                return Err(ToolError::not_found("task", &id));
            }

            // Read file content to extract body
            let content = std::fs::read_to_string(&meta.path)
                .map_err(|e| ToolError::internal(format!("Failed to read task file: {}", e)))?;
            let (_, body) = crate::parser::extract_frontmatter(&content);

            Ok(json!({
                "id": meta.id,
                "title": meta.title,
                "status": format!("{:?}", meta.status).to_lowercase(),
                "priority": meta.priority.as_ref().map(|p| format!("{:?}", p).to_lowercase()),
                "tags": meta.tags,
                "assignee": meta.assignee,
                "acceptance_criteria": meta.acceptance_criteria.iter().map(|ac| json!({
                    "text": ac.text,
                    "checked": ac.checked
                })).collect::<Vec<_>>(),
                "content": body,
            }))
        }),
    );

    // ── wm_task.update ──────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_schema(
        "wm_task.update",
        "Update a task. Only updates pages with type: task.",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Task page ID" },
                "title": { "type": "string", "description": "New title" },
                "status": { "type": "string", "description": "New status: todo/in_progress/done/blocked" },
                "priority": { "type": "string", "description": "New priority: low/medium/high/urgent" },
                "tags": { "type": "array", "items": { "type": "string" }, "description": "New tags" },
                "acceptance_criteria": { "type": "array", "items": { "type": "string" }, "description": "New acceptance criteria" },
                "assignee": { "type": "string", "description": "New assignee" },
                "content": { "type": "string", "description": "New content" }
            },
            "required": ["id"]
        }),
        Arc::new(move |params| {
            // Clone params before consuming for ToolArgs
            let args = ToolArgs::new(params.clone());
            let id = args.require_string("id")?;

            // Verify it's a task
            let snapshot = e.graph.load();
            let index = &snapshot.1;
            let node_idx = index
                .get(&id)
                .ok_or_else(|| ToolError::not_found("task", &id))?;
            let meta = &snapshot.0[*node_idx];

            if meta.page_type != PageType::Task {
                return Err(ToolError::not_found("task", &id));
            }

            // Build update JSON from provided params
            let mut update = serde_json::Map::new();

            if let Some(title) = args.optional_string("title") {
                update.insert("title".to_string(), json!(title));
            }
            if let Some(status) = args.optional_string("status") {
                update.insert("status".to_string(), json!(status));
            }
            if let Some(priority) = args.optional_string("priority") {
                update.insert("priority".to_string(), json!(priority));
            }
            if let Some(assignee) = args.optional_string("assignee") {
                update.insert("assignee".to_string(), json!(assignee));
            }
            if params.get("tags").and_then(|v| v.as_array()).is_some() {
                let tags: Vec<String> = args.optional_string_array("tags");
                update.insert("tags".to_string(), json!(tags));
            }
            if params.get("acceptance_criteria").and_then(|v| v.as_array()).is_some() {
                let criteria: Vec<serde_json::Value> = args
                    .optional_string_array("acceptance_criteria")
                    .iter()
                    .map(|text| json!({"text": text, "checked": false}))
                    .collect();
                update.insert("acceptance_criteria".to_string(), json!(criteria));
            }
            if let Some(content) = args.optional_text("content") {
                update.insert("content".to_string(), json!(content));
            }

            page::update_page(&e, &id, &json!(update))?;

            Ok(json!({
                "id": id,
                "status": "updated"
            }))
        }),
    );

    // ── wm_task.delete ──────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_schema(
        "wm_task.delete",
        "Delete a task by ID. Only allows deletion of pages with type: task.",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Task page ID to delete" }
            },
            "required": ["id"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;

            let snapshot = e.graph.load();
            let index = &snapshot.1;
            let node_idx = index
                .get(&id)
                .ok_or_else(|| ToolError::not_found("task", &id))?;
            let meta = &snapshot.0[*node_idx];

            if meta.page_type != PageType::Task {
                return Err(ToolError::not_found("task", &id));
            }

            let file_path = &meta.path;
            if file_path.exists() {
                std::fs::remove_file(file_path).map_err(|e| {
                    ToolError::internal(format!("Failed to delete {}: {}", file_path.display(), e))
                })?;
            }

            e.stale_flag
                .store(true, std::sync::atomic::Ordering::Release);
            Ok(json!({ "id": id, "status": "deleted" }))
        }),
    );

    // ── Existing tools ───────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_schema(
        "wm_task.check_ac",
        "Check an acceptance criterion",
        json!({
            "type": "object",
            "properties": {
                "task_id": { "type": "string", "description": "Task page ID" },
                "index": { "type": "integer", "description": "Index of the acceptance criterion to check" }
            },
            "required": ["task_id", "index"]
        }),
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
    registry.register_with_schema(
        "wm_task.uncheck_ac",
        "Uncheck an acceptance criterion",
        json!({
            "type": "object",
            "properties": {
                "task_id": { "type": "string", "description": "Task page ID" },
                "index": { "type": "integer", "description": "Index of the acceptance criterion to uncheck" }
            },
            "required": ["task_id", "index"]
        }),
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
    registry.register_with_schema("wm_task.board", "Task board grouped by status", json!({
        "type": "object",
        "properties": {}
    }), Arc::new(move |_params| {
        let board = crate::task::task_board(&e);
        Ok(serde_json::json!(board))
    }));

    let e = engine.clone();
    registry.register_with_schema(
        "wm_task.list",
        "List tasks with optional filters (status, label, limit)",
        json!({
            "type": "object",
            "properties": {
                "status": { "type": "string", "description": "Filter by status: todo/in_progress/done/cancelled" },
                "label": { "type": "string", "description": "Filter by label/tag" },
                "limit": { "type": "integer", "description": "Max results", "default": 50 }
            }
        }),
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
