use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::engine::{EngineState, PageStatus, PageType};
use crate::error::ToolError;
use crate::mcp::transport::ToolRegistry;
use crate::mcp::typed::TypedRegister;
use crate::page;
use crate::parser;

// ─── Input / Output types ───────────────────────────────────

#[derive(Deserialize, JsonSchema)]
struct WmTaskCreateInput {
    #[schemars(description = "Task page ID")]
    id: String,
    #[schemars(description = "Task title")]
    title: String,
    #[schemars(description = "Task status: draft/todo/in_progress/in_review/blocked/done/reviewed/approved/superseded/cancelled")]
    status: Option<String>,
    #[schemars(description = "Priority: low/medium/high/urgent")]
    priority: Option<String>,
    #[schemars(description = "Tags")]
    tags: Option<Vec<String>>,
    #[schemars(description = "Acceptance criteria")]
    acceptance_criteria: Option<Vec<String>>,
    #[schemars(description = "Assignee name")]
    assignee: Option<String>,
    #[schemars(description = "Task description content")]
    content: Option<String>,
}

#[derive(Serialize)]
struct WmTaskCreateOutput {
    id: String,
    title: String,
    status: String,
    priority: String,
    tags: Vec<String>,
    acceptance_criteria: Vec<String>,
    assignee: Option<String>,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Deserialize, JsonSchema)]
struct WmTaskGetInput {
    #[schemars(description = "Task page ID")]
    id: String,
}

#[derive(Deserialize, JsonSchema)]
struct WmTaskUpdateInput {
    #[schemars(description = "Task page ID")]
    id: String,
    #[schemars(description = "New title")]
    title: Option<String>,
    #[schemars(description = "New status: draft/todo/in_progress/in_review/blocked/done/reviewed/approved/superseded/cancelled — validated against state machine")]
    status: Option<String>,
    #[schemars(description = "New priority: low/medium/high/urgent")]
    priority: Option<String>,
    #[schemars(description = "New tags")]
    tags: Option<Vec<String>>,
    #[schemars(description = "New acceptance criteria")]
    acceptance_criteria: Option<Vec<String>>,
    #[schemars(description = "New assignee")]
    assignee: Option<String>,
    #[schemars(description = "New content")]
    content: Option<String>,
    #[schemars(description = "Implementation notes (replaces existing)")]
    notes: Option<String>,
    #[schemars(description = "Append to implementation notes (mode: append — adds newline + content)")]
    append_notes: Option<String>,
}

#[derive(Serialize)]
struct WmTaskUpdateOutput {
    id: String,
    status: String,
}

#[derive(Deserialize, JsonSchema)]
struct WmTaskDeleteInput {
    #[schemars(description = "Task page ID to delete")]
    id: String,
}

#[derive(Serialize)]
struct WmTaskDeleteOutput {
    id: String,
    status: String,
}

#[derive(Deserialize, JsonSchema)]
struct WmTaskCheckAcInput {
    #[schemars(description = "Task page ID")]
    id: String,
    #[schemars(description = "1-based indices of ACs to check")]
    criteria: Vec<u64>,
}

#[derive(Deserialize, JsonSchema)]
struct WmTaskUncheckAcInput {
    #[schemars(description = "Task page ID")]
    id: String,
    #[schemars(description = "1-based indices of ACs to uncheck")]
    criteria: Vec<u64>,
}

#[derive(Deserialize, JsonSchema)]
struct WmTaskBoardInput {}

#[derive(Deserialize, JsonSchema)]
struct WmTaskListInput {
    #[schemars(description = "Filter by status: todo/in_progress/done/cancelled")]
    status: Option<String>,
    #[schemars(description = "Filter by label/tag")]
    label: Option<String>,
    #[schemars(description = "Max results")]
    limit: Option<usize>,
}

/// Register task tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    // ── wm_task.create ──────────────────────────────────────────
    let e = engine.clone();
    registry.register_write(
        "wm_task.create",
        "Create a task wiki page. Sets type: task automatically.",
        move |input: WmTaskCreateInput| {
            let id = input.id;
            let title = input.title;
            let status = input.status.unwrap_or_else(|| "todo".to_string());
            let priority = input.priority.unwrap_or_else(|| "medium".to_string());
            let tags = input.tags.unwrap_or_default();
            let acceptance_criteria = input.acceptance_criteria.unwrap_or_default();
            let assignee = input.assignee;
            let content = input
                .content
                .as_deref()
                .map(crate::util::unescape_text)
                .unwrap_or_default();

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

            Ok(WmTaskCreateOutput {
                id: page_id,
                title,
                status,
                priority,
                tags,
                acceptance_criteria,
                assignee,
                type_: "task".to_string(),
            })
        },
    );

    // ── wm_task.get ─────────────────────────────────────────────
    let e = engine.clone();
    registry.register_read(
        "wm_task.get",
        "Get a task by ID. Only returns pages with type: task.",
        move |input: WmTaskGetInput| {
            let id = input.id;

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
        },
    );

    // ── wm_task.update ──────────────────────────────────────────
    let e = engine.clone();
    registry.register_write(
        "wm_task.update",
        "Update a task. Only updates pages with type: task.",
        move |input: WmTaskUpdateInput| {
            let id = input.id;

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

            if let Some(title) = input.title {
                update.insert("title".to_string(), json!(title));
            }
            if let Some(status) = input.status {
                // Validate state transition
                let current_status = &meta.status;
                let new_status = parser::parse_page_status(&status);
                if let Err(msg) = current_status.can_transition_to(&new_status) {
                    return Err(ToolError::internal(msg));
                }
                update.insert("status".to_string(), json!(status));
            }
            if let Some(priority) = input.priority {
                update.insert("priority".to_string(), json!(priority));
            }
            if let Some(assignee) = input.assignee {
                update.insert("assignee".to_string(), json!(assignee));
            }
            if let Some(tags) = input.tags {
                update.insert("tags".to_string(), json!(tags));
            }
            if let Some(criteria) = input.acceptance_criteria {
                let criteria: Vec<serde_json::Value> = criteria
                    .iter()
                    .map(|text| json!({"text": text, "checked": false}))
                    .collect();
                update.insert("acceptance_criteria".to_string(), json!(criteria));
            }
            if let Some(content) = input.content {
                update.insert(
                    "content".to_string(),
                    json!(crate::util::unescape_text(&content)),
                );
            }
            if let Some(notes) = input.notes {
                update.insert("implementation_notes".to_string(), json!(notes));
            }
            if let Some(append) = input.append_notes {
                update.insert("append_notes".to_string(), json!(append));
            }

            page::update_page(&e, &id, &json!(update))?;

            Ok(WmTaskUpdateOutput {
                id,
                status: "updated".to_string(),
            })
        },
    );

    // ── wm_task.delete ──────────────────────────────────────────
    let e = engine.clone();
    registry.register_admin(
        "wm_task.delete",
        "Delete a task by ID. Only allows deletion of pages with type: task.",
        move |input: WmTaskDeleteInput| {
            let id = input.id;

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
            Ok(WmTaskDeleteOutput {
                id,
                status: "deleted".to_string(),
            })
        },
    );

    // ── wm_task.check_ac ────────────────────────────────────────
    let e = engine.clone();
    registry.register_write(
        "wm_task.check_ac",
        "Check acceptance criteria by index",
        move |input: WmTaskCheckAcInput| {
            let id = input.id;
            let indices = input.criteria;
            let update = serde_json::json!({ "checked_ac": indices });
            page::update_page(&e, &id, &update)?;
            Ok(serde_json::json!({ "id": id, "checked": indices }))
        },
    );

    // ── wm_task.uncheck_ac ──────────────────────────────────────
    let e = engine.clone();
    registry.register_write(
        "wm_task.uncheck_ac",
        "Uncheck acceptance criteria by index",
        move |input: WmTaskUncheckAcInput| {
            let id = input.id;
            let indices = input.criteria;
            let update = serde_json::json!({ "unchecked_ac": indices });
            page::update_page(&e, &id, &update)?;
            Ok(serde_json::json!({ "id": id, "unchecked": indices }))
        },
    );

    // ── wm_task.board ───────────────────────────────────────────
    let e = engine.clone();
    registry.register_read(
        "wm_task.board",
        "Task board grouped by status — returns full task detail per column",
        move |_input: WmTaskBoardInput| {
            let snapshot = e.graph.load();
            let graph = &snapshot.0;

            let all_statuses = PageStatus::task_board_columns();

            // Initialize buckets for each status
            let mut buckets: std::collections::HashMap<String, Vec<serde_json::Value>> =
                std::collections::HashMap::new();
            for status in &all_statuses {
                buckets.insert(status.as_str().to_string(), Vec::new());
            }

            for idx in graph.node_indices() {
                let meta = &graph[idx];
                if meta.page_type != PageType::Task {
                    continue;
                }

                // Read file for description and time_spent
                let (description, time_spent) = read_task_file_detail(&meta.path);

                let task = serde_json::json!({
                    "id": meta.id,
                    "title": meta.title,
                    "description": description,
                    "status": meta.status.as_str(),
                    "priority": meta.priority.as_ref().map(|p| p.as_str()),
                    "labels": meta.tags,
                    "createdAt": meta.created_at,
                    "updatedAt": meta.updated_at,
                    "acceptanceCriteria": meta.acceptance_criteria.iter().map(|ac| serde_json::json!({
                        "text": ac.text,
                        "completed": ac.checked
                    })).collect::<Vec<_>>(),
                    "timeSpent": time_spent,
                });

                let key = meta.status.as_str().to_string();
                buckets.entry(key).or_default().push(task);
            }

            let mut columns = serde_json::Map::new();
            let mut column_order: Vec<String> = Vec::new();
            let mut counts = serde_json::Map::new();
            for status in &all_statuses {
                let key = status.as_str().to_string();
                let tasks = buckets.remove(&key).unwrap_or_default();
                let count = tasks.len();
                columns.insert(key.clone(), serde_json::Value::Array(tasks));
                column_order.push(key.clone());
                counts.insert(key, serde_json::Value::Number(count.into()));
            }

            Ok(serde_json::json!({ "columns": columns, "columnOrder": column_order, "counts": counts }))
        },
    );

    // ── wm_task.list ────────────────────────────────────────────
    let e = engine.clone();
    registry.register_read(
        "wm_task.list",
        "List tasks with optional filters (status, label, limit)",
        move |input: WmTaskListInput| {
            let filter_status = input.status.map(|s| s.to_lowercase());
            let filter_label = input.label.map(|s| s.to_lowercase());
            let limit = input.limit.unwrap_or(50);

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
                    "status": meta.status.as_str(),
                    "priority": meta.priority.as_ref().map(|p| p.as_str()),
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
        },
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

/// Read a task page file and extract the description (first non-empty body line)
/// and time_spent (parsed to total minutes).
fn read_task_file_detail(path: &std::path::Path) -> (String, u64) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (String::new(), 0),
    };

    let (fm, body) = crate::parser::extract_frontmatter(&content);

    // First non-empty line of body as description
    let description = body
        .lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .unwrap_or_default();

    // Parse time_spent from frontmatter (format: "Xh Ym" or "Xm" or "Xh")
    let time_spent = fm
        .as_ref()
        .and_then(|f| f.time_spent.as_deref())
        .map(parse_time_spent_to_minutes)
        .unwrap_or(0);

    (description, time_spent)
}

/// Parse a time_spent string like "2h 30m" or "45m" or "1h" into total minutes.
fn parse_time_spent_to_minutes(s: &str) -> u64 {
    let s = s.trim().to_lowercase();
    let mut total: u64 = 0;

    // Match "Xh" pattern
    if let Some(pos) = s.find('h') {
        if let Ok(hours) = s[..pos].trim().parse::<u64>() {
            total += hours * 60;
        }
    }

    // Match "Xm" pattern
    if let Some(pos) = s.find('m') {
        // Find the start of the minutes part (after any 'h')
        let start = s.rfind('h').map(|p| p + 1).unwrap_or(0);
        if let Ok(mins) = s[start..pos].trim().parse::<u64>() {
            total += mins;
        }
    }

    total
}
