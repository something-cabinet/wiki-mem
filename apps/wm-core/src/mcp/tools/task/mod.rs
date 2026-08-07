pub mod action;
pub mod output;

pub use action::*;
pub use output::*;

use crate::engine::{PageStatus, PageType, Priority};
use crate::mcp::prelude::*;
use serde_json::json;
use wm_constants::*;

use crate::page;
use crate::version::{FieldChange, VersionStore};

struct CreateTaskParams {
    engine: Arc<EngineState>,
    title: String,
    description: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    assignee: Option<String>,
    labels: Option<Vec<String>>,
    parent: Option<String>,
    spec: Option<String>,
    estimate: Option<u32>,
    acceptance_criteria: Option<Vec<String>>,
}

struct UpdateTaskParams {
    engine: Arc<EngineState>,
    id: String,
    title: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    assignee: Option<String>,
    labels: Option<Vec<String>>,
    description: Option<String>,
    implementation_plan: Option<String>,
    implementation_notes: Option<String>,
    append_notes: Option<String>,
    acceptance_criteria: Option<Vec<String>>,
}

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    registry.register_typed(
        "wm_task",
        "Task operations: board, list, create, get, update, delete, check_ac, uncheck_ac, subtask",
        move |input: WmTaskAction| -> Result<serde_json::Value, ToolError> {
            match input {
                WmTaskAction::Board {} => handle_board(&engine),
                WmTaskAction::List {
                    status,
                    priority,
                    assignee,
                    label,
                    limit,
                } => handle_list(&engine, status, priority, assignee, label, limit),
                WmTaskAction::Create {
                    title,
                    description,
                    status,
                    priority,
                    assignee,
                    labels,
                    parent,
                    spec,
                    estimate,
                    acceptance_criteria,
                } => handle_create(CreateTaskParams {
                    engine: engine.clone(),
                    title,
                    description,
                    status,
                    priority,
                    assignee,
                    labels,
                    parent,
                    spec,
                    estimate,
                    acceptance_criteria,
                }),
                WmTaskAction::Get { id } => handle_get(&engine, id),
                WmTaskAction::Update {
                    id,
                    title,
                    status,
                    priority,
                    assignee,
                    labels,
                    description,
                    implementation_plan,
                    implementation_notes,
                    append_notes,
                    acceptance_criteria,
                } => handle_update(UpdateTaskParams {
                    engine: engine.clone(),
                    id,
                    title,
                    status,
                    priority,
                    assignee,
                    labels,
                    description,
                    implementation_plan,
                    implementation_notes,
                    append_notes,
                    acceptance_criteria,
                }),
                WmTaskAction::Delete { id } => handle_delete(&engine, id),
                WmTaskAction::CheckAc { id, index } => handle_check_ac(&engine, id, index),
                WmTaskAction::UncheckAc { id, index } => handle_uncheck_ac(&engine, id, index),
                WmTaskAction::Subtask {
                    id,
                    title,
                    status,
                    priority,
                } => handle_subtask(&engine, id, title, status, priority),
            }
        },
    );
}

fn handle_board(engine: &Arc<EngineState>) -> Result<serde_json::Value, ToolError> {
    let snapshot = engine.graph.load();
    let graph = &snapshot.0;

    let cfg = engine
        .config
        .read()
        .map_err(|_| ToolError::lock_poisoned("config"))?;
    let status_colors = cfg.status_colors.colors.clone();
    let visible_columns = cfg.visible_columns.clone();
    drop(cfg);

    let all_statuses = PageStatus::task_board_columns();

    let statuses: Vec<PageStatus> = if let Some(ref visible) = visible_columns {
        all_statuses
            .into_iter()
            .filter(|s| visible.contains(&s.as_str().to_string()))
            .collect()
    } else {
        all_statuses
    };

    let mut subtask_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for idx in graph.node_indices() {
        let meta = &graph[idx];
        if meta.page_type == PageType::Task {
            if let Some(ref parent) = meta.parent {
                let counter = subtask_counts.entry(parent.clone()).or_insert(0);
                *counter = counter.wrapping_add(1);
            }
        }
    }

    let mut buckets: std::collections::HashMap<String, Vec<serde_json::Value>> =
        std::collections::HashMap::new();
    for status in &statuses {
        buckets.insert(status.as_str().to_string(), Vec::new());
    }

    for idx in graph.node_indices() {
        let meta = &graph[idx];
        if meta.page_type != PageType::Task {
            continue;
        }

        let (description, time_spent) = read_task_file_detail(&meta.path);

        let task = json!({
            "id": meta.id,
            "title": meta.title,
            "description": description,
            "status": meta.status.as_str(),
            "priority": meta.priority.as_ref().map(|p| p.as_str()),
            "labels": meta.tags,
            "createdAt": meta.created_at,
            "updatedAt": meta.updated_at,
            "acceptanceCriteria": meta.task_data.as_ref().map(|td| {
                td.acceptance_criteria.iter().map(|ac| json!({
                    "text": &ac.text,
                    "completed": ac.checked
                })).collect::<Vec<_>>()
            }).unwrap_or_default(),
            "timeSpent": time_spent,
            "subtaskCount": subtask_counts.get(&meta.id).copied().unwrap_or(0),
        });

        let key = meta.status.as_str().to_string();
        buckets.entry(key).or_default().push(task);
    }

    let mut columns = serde_json::Map::new();
    let mut column_order: Vec<String> = Vec::new();
    let mut counts = serde_json::Map::new();
    for status in &statuses {
        let key = status.as_str().to_string();
        let tasks = buckets.remove(&key).unwrap_or_default();
        let count = tasks.len();
        columns.insert(key.clone(), serde_json::Value::Array(tasks));
        column_order.push(key.clone());
        counts.insert(key, serde_json::Value::Number(count.into()));
    }

    Ok(
        json!({ "columns": columns, "columnOrder": column_order, "counts": counts, "colors": status_colors }),
    )
}

fn handle_list(
    engine: &Arc<EngineState>,
    status: Option<String>,
    _priority: Option<String>,
    _assignee: Option<String>,
    label: Option<String>,
    limit: Option<usize>,
) -> Result<serde_json::Value, ToolError> {
    let filter_status = status.map(|s| s.to_string());
    let filter_label = label.map(|s| s.to_lowercase());
    let limit = limit.unwrap_or(50);

    let snapshot = engine.graph.load();
    let graph = &snapshot.0;

    let mut tasks: Vec<serde_json::Value> = Vec::new();

    for idx in graph.node_indices() {
        let meta = &graph[idx];
        if meta.page_type != PageType::Task {
            continue;
        }

        if let Some(ref status) = filter_status {
            let task_status = meta.status.as_str();
            if task_status != *status {
                continue;
            }
        }

        if let Some(ref label) = filter_label {
            let has_label = meta.tags.iter().any(|t| t.to_lowercase() == *label);
            if !has_label {
                continue;
            }
        }

        let description = extract_task_description(&meta.path);

        tasks.push(json!({
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

    Ok(json!({
        "tasks": tasks,
        "total": tasks.len(),
    }))
}

fn handle_create(params: CreateTaskParams) -> Result<serde_json::Value, ToolError> {
    let CreateTaskParams {
        engine,
        title,
        description,
        status,
        priority,
        assignee,
        labels,
        parent,
        spec,
        estimate,
        acceptance_criteria,
    } = params;

    let status_val = if let Some(ref s) = status {
        let ps: PageStatus = serde_json::from_value(serde_json::Value::String(s.clone()))
            .map_err(|e| ToolError::invalid_params(format!("Invalid status '{}': {}", s, e)))?;
        if !PageType::Task.allowed_statuses().contains(&ps) {
            return Err(ToolError::invalid_params(format!(
                "Invalid status '{}' for task page. Allowed: {}",
                ps.as_str(),
                PageType::Task
                    .allowed_statuses()
                    .iter()
                    .map(|s| s.as_str())
                    .fold(String::new(), |mut acc, s| {
                        if !acc.is_empty() {
                            acc.push_str(", ");
                        }
                        acc.push_str(s);
                        acc
                    },)
            )));
        }
        ps
    } else {
        PageStatus::Todo
    };

    let priority_val = if let Some(ref p) = priority {
        serde_json::from_value(serde_json::Value::String(p.clone()))
            .map_err(|e| ToolError::invalid_params(format!("Invalid priority '{}': {}", p, e)))?
    } else {
        Priority::Medium
    };

    let tags = labels.unwrap_or_default();
    let content = description
        .as_deref()
        .map(crate::util::unescape_text)
        .unwrap_or_default();

    let slug = title
        .to_lowercase()
        .replace(' ', "-")
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "");
    let task_id = format!("wiki:tasks:{}", slug);
    let mut frontmatter = format!(
        "title: {}\ntype: task\nid: {}\nstatus: {}\npriority: {}\n",
        title, task_id, status_val, priority_val
    );

    if !tags.is_empty() {
        frontmatter.push_str(&format!("tags: [{}]\n", tags.join(", ")));
    }

    if let Some(ref assignee) = assignee {
        frontmatter.push_str(&format!("assignee: {}\n", assignee));
    }

    if let Some(ref parent) = parent {
        frontmatter.push_str(&format!("parent: {}\n", parent));
    }

    if let Some(ref spec) = spec {
        frontmatter.push_str(&format!("spec: {}\n", spec));
    }

    if let Some(estimate) = estimate {
        frontmatter.push_str(&format!("estimate: {}\n", estimate));
    }

    if let Some(ref ac_list) = acceptance_criteria {
        if !ac_list.is_empty() {
            frontmatter.push_str("acceptance_criteria:\n");
            for ac in ac_list {
                frontmatter.push_str(&format!("  - text: \"{}\"\n", ac.replace('\"', "\\\"")));
            }
        }
    }

    let slug = title
        .to_lowercase()
        .replace(' ', "-")
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "");
    let path = format!("tasks/{}", slug);
    let page_id = page::create_page(&engine, &path, &frontmatter, &content)?;

    let response_acs = acceptance_criteria.unwrap_or_default();

    Ok(serde_json::to_value(WmTaskCreateOutput {
        id: page_id,
        title,
        status: status_val.to_string(),
        priority: priority_val.to_string(),
        tags,
        acceptance_criteria: response_acs,
        assignee,
        type_: "task".into(),
    })
    .unwrap_or(serde_json::Value::Null))
}

fn handle_get(engine: &Arc<EngineState>, id: String) -> Result<serde_json::Value, ToolError> {
    let meta = crate::page::services::page_crud_service::resolve_page_meta(
        engine,
        &id,
        &crate::page_repo::FsPageRepo,
    )?;

    if meta.page_type != PageType::Task {
        return Err(ToolError::not_found("task", &id));
    }

    let content = std::fs::read_to_string(&meta.path)
        .map_err(|e| ToolError::internal(format!("Failed to read task file: {}", e)))?;
    let (_, body) = crate::parser::extract_frontmatter(&content);

    let mut subtasks = Vec::new();
    let snapshot = engine.graph.load();
    for sub_idx in snapshot.0.node_indices() {
        let sub_meta = &snapshot.0[sub_idx];
        if sub_meta.page_type == PageType::Task && sub_meta.parent.as_deref() == Some(&id) {
            subtasks.push(json!({
                "id": sub_meta.id,
                "title": sub_meta.title,
                "status": sub_meta.status.as_str(),
                "priority": sub_meta.priority.as_ref().map(|p| format!("{:?}", p).to_lowercase()),
            }));
        }
    }

    Ok(json!({
        "id": meta.id,
        "title": meta.title,
        "status": meta.status.as_str(),
        "priority": meta.priority.as_ref().map(|p| format!("{:?}", p).to_lowercase()),
        "tags": meta.tags,
        "assignee": meta.assignee,
        "acceptance_criteria": meta.task_data.as_ref().map(|td| {
            td.acceptance_criteria.iter().map(|ac| json!({
                "text": &ac.text,
                "checked": ac.checked
            })).collect::<Vec<_>>()
        }).unwrap_or_default(),
        "content": body,
        "subtasks": subtasks,
    }))
}

fn handle_update(params: UpdateTaskParams) -> Result<serde_json::Value, ToolError> {
    let UpdateTaskParams {
        engine,
        id,
        title,
        status,
        priority,
        assignee,
        labels,
        description,
        implementation_plan,
        implementation_notes,
        append_notes,
        acceptance_criteria,
    } = params;

    let meta = crate::page::services::page_crud_service::resolve_page_meta(
        &engine,
        &id,
        &crate::page_repo::FsPageRepo,
    )?;

    if meta.page_type != PageType::Task {
        return Err(ToolError::not_found("task", &id));
    }

    if let Some(ref s) = status {
        let parsed: PageStatus = serde_json::from_value(serde_json::Value::String(s.clone()))
            .map_err(|e| ToolError::invalid_params(format!("Invalid status '{}': {}", s, e)))?;
        if !PageType::Task.allowed_statuses().contains(&parsed) {
            return Err(ToolError::invalid_params(format!(
                "Invalid status '{}' for task page. Allowed: {}",
                parsed.as_str(),
                PageType::Task
                    .allowed_statuses()
                    .iter()
                    .map(|s| s.as_str())
                    .fold(String::new(), |mut acc, s| {
                        if !acc.is_empty() {
                            acc.push_str(", ");
                        }
                        acc.push_str(s);
                        acc
                    },)
            )));
        }
        let current_status = &meta.status;
        if let Err(msg) = current_status.can_transition_to(&parsed) {
            return Err(ToolError::internal(msg));
        }
    }

    let root = engine
        .project_root
        .read()
        .map(|r| r.clone())
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    let store = VersionStore::new(root.join(WM_DIR));

    let file_path = &meta.path;
    let old_content = std::fs::read_to_string(file_path).unwrap_or_default();
    let (old_fm, _old_body) = crate::parser::extract_frontmatter(&old_content);

    let mut changes: Vec<FieldChange> = Vec::new();

    if let Some(ref new_title) = title {
        let old_val = old_fm.as_ref().and_then(|fm| fm.title.as_deref());
        changes.push(FieldChange {
            field: "title".into(),
            old_value: old_val.map(|s| serde_json::Value::String(s.to_string())),
            new_value: Some(serde_json::Value::String(new_title.clone())),
        });
    }
    if status.is_some() {
        let old_val = old_fm.as_ref().and_then(|fm| fm.status.as_deref());
        changes.push(FieldChange {
            field: "status".into(),
            old_value: old_val.map(|s| serde_json::Value::String(s.to_string())),
            new_value: status.clone().map(serde_json::Value::String),
        });
    }
    if priority.is_some() {
        let old_val = old_fm.as_ref().and_then(|fm| fm.priority.as_deref());
        changes.push(FieldChange {
            field: "priority".into(),
            old_value: old_val.map(|s| serde_json::Value::String(s.to_string())),
            new_value: priority.clone().map(serde_json::Value::String),
        });
    }
    if assignee.is_some() {
        let old_val = old_fm.as_ref().and_then(|fm| fm.assignee.as_deref());
        changes.push(FieldChange {
            field: "assignee".into(),
            old_value: old_val.map(|s| serde_json::Value::String(s.to_string())),
            new_value: assignee.clone().map(serde_json::Value::String),
        });
    }
    if labels.is_some() {
        let old_val = old_fm.as_ref().map(|fm| fm.tags.clone());
        changes.push(FieldChange {
            field: "tags".into(),
            old_value: old_val.map(|v| serde_json::to_value(v).unwrap_or_default()),
            new_value: labels
                .clone()
                .map(|v| serde_json::to_value(v).unwrap_or_default()),
        });
    }
    if description.is_some() {
        let old_val = Some(_old_body.trim().to_string());
        changes.push(FieldChange {
            field: "content".into(),
            old_value: old_val.map(serde_json::Value::String),
            new_value: description
                .clone()
                .map(|c| serde_json::Value::String(crate::util::unescape_text(&c))),
        });
    }
    if implementation_plan.is_some() {
        changes.push(FieldChange {
            field: "implementation_plan".into(),
            old_value: meta
                .task_data
                .as_ref()
                .and_then(|d| d.implementation_plan.as_ref())
                .map(|s| serde_json::Value::String(s.clone())),
            new_value: implementation_plan.clone().map(serde_json::Value::String),
        });
    }
    if implementation_notes.is_some() || append_notes.is_some() {
        let old_val = meta
            .task_data
            .as_ref()
            .and_then(|d| d.implementation_notes.as_ref());
        let merged = match (&implementation_notes, &append_notes, old_val) {
            (Some(new_notes), _, _) => new_notes.clone(),
            (_, Some(append), Some(existing)) => format!("{}\n{}", existing, append),
            (_, Some(append), None) => append.clone(),
            _ => String::new(),
        };
        changes.push(FieldChange {
            field: "implementation_notes".into(),
            old_value: old_val.map(|s| serde_json::Value::String(s.clone())),
            new_value: Some(serde_json::Value::String(merged)),
        });
    }

    store.save_task_version(&id, changes)?;

    let params = page::PageUpdateParams {
        title,
        status: status.map(|s| s.to_string()),
        priority: priority.map(|p| p.to_string()),
        assignee,
        tags: labels,
        content: description.map(|c| crate::util::unescape_text(&c)),
        implementation_plan,
        implementation_notes,
        append_notes,
        acceptance_criteria: acceptance_criteria.map(|acs| {
            acs.into_iter()
                .map(|text| crate::engine::AcceptanceCriterion {
                    text,
                    checked: false,
                })
                .collect()
        }),
        ..Default::default()
    };
    page::update_page(&engine, &id, &params)?;

    Ok(serde_json::to_value(WmTaskUpdateOutput {
        id,
        status: "updated".into(),
    })
    .unwrap_or(serde_json::Value::Null))
}

fn handle_delete(engine: &Arc<EngineState>, id: String) -> Result<serde_json::Value, ToolError> {
    let meta = crate::page::services::page_crud_service::resolve_page_meta(
        engine,
        &id,
        &crate::page_repo::FsPageRepo,
    )?;

    if meta.page_type != PageType::Task {
        return Err(ToolError::not_found("task", &id));
    }

    let file_path = &meta.path;
    if file_path.exists() {
        std::fs::remove_file(file_path).map_err(|e| {
            ToolError::internal(format!("Failed to delete {}: {}", file_path.display(), e))
        })?;
    }

    engine
        .stale_flag
        .store(true, std::sync::atomic::Ordering::Release);
    Ok(serde_json::to_value(WmTaskDeleteOutput {
        id,
        status: "deleted".into(),
    })
    .unwrap_or(serde_json::Value::Null))
}

fn handle_check_ac(
    engine: &Arc<EngineState>,
    id: String,
    index: usize,
) -> Result<serde_json::Value, ToolError> {
    let indices = vec![u64::try_from(index).unwrap_or(u64::MAX)];
    let params = page::PageUpdateParams {
        checked_ac: Some(indices.clone()),
        ..Default::default()
    };
    page::update_page(engine, &id, &params)?;
    Ok(json!({ "id": id, "checked": indices }))
}

fn handle_uncheck_ac(
    engine: &Arc<EngineState>,
    id: String,
    index: usize,
) -> Result<serde_json::Value, ToolError> {
    let indices = vec![u64::try_from(index).unwrap_or(u64::MAX)];
    let params = page::PageUpdateParams {
        unchecked_ac: Some(indices.clone()),
        ..Default::default()
    };
    page::update_page(engine, &id, &params)?;
    Ok(json!({ "id": id, "unchecked": indices }))
}

fn handle_subtask(
    engine: &Arc<EngineState>,
    parent_id: String,
    title: String,
    status: Option<String>,
    _priority: Option<String>,
) -> Result<serde_json::Value, ToolError> {
    let content = String::new();

    let snapshot = engine.graph.load();
    let index = &snapshot.1;
    let parent_idx = index
        .get(&parent_id)
        .ok_or_else(|| ToolError::not_found("parent task", &parent_id))?;
    let parent_meta = &snapshot.0[*parent_idx];
    if parent_meta.page_type != PageType::Task {
        return Err(ToolError::internal(format!(
            "Cannot create subtask: '{}' is not a task",
            parent_id
        )));
    }

    let slug = title
        .to_lowercase()
        .replace(' ', "-")
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "");
    let subtask_id = format!("{}/{}", parent_id.trim_end_matches(".md"), slug);

    let tags = parent_meta.tags.clone();

    let status_val = if let Some(ref s) = status {
        let ps: PageStatus = serde_json::from_value(serde_json::Value::String(s.clone()))
            .map_err(|e| ToolError::invalid_params(format!("Invalid status '{}': {}", s, e)))?;
        if !PageType::Task.allowed_statuses().contains(&ps) {
            return Err(ToolError::invalid_params(format!(
                "Invalid status '{}' for task page. Allowed: {}",
                ps.as_str(),
                PageType::Task
                    .allowed_statuses()
                    .iter()
                    .map(|s| s.as_str())
                    .fold(String::new(), |mut acc, s| {
                        if !acc.is_empty() {
                            acc.push_str(", ");
                        }
                        acc.push_str(s);
                        acc
                    },)
            )));
        }
        ps
    } else {
        PageStatus::Todo
    };

    let mut frontmatter = format!(
        "title: {}\ntype: task\nparent: {}\nstatus: {}\n",
        title, parent_id, status_val
    );
    if !tags.is_empty() {
        frontmatter.push_str(&format!("tags: [{}]\n", tags.join(", ")));
    }

    drop(snapshot); // release graph lock before file write

    let id = crate::page::create_page(engine, &subtask_id, &frontmatter, &content)?;

    engine
        .stale_flag
        .store(true, std::sync::atomic::Ordering::Release);

    Ok(serde_json::to_value(WmTaskCreateOutput {
        id,
        title,
        status: status_val.to_string(),
        priority: "medium".into(),
        tags,
        acceptance_criteria: Vec::new(),
        assignee: None,
        type_: "task".into(),
    })
    .unwrap_or(serde_json::Value::Null))
}

fn extract_task_description(path: &std::path::Path) -> String {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };

    let body = if let Some(end) = content.find("\n---") {
        let after = &content[end.wrapping_add(4)..];
        if let Some(stripped) = after.strip_prefix('\n') {
            stripped
        } else {
            after
        }
    } else {
        &content
    };

    let body = body.trim();

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

fn read_task_file_detail(path: &std::path::Path) -> (String, u64) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (String::new(), 0),
    };

    let (fm, body) = crate::parser::extract_frontmatter(&content);

    let description = body
        .lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .unwrap_or_default();

    let time_spent = fm
        .as_ref()
        .and_then(|f| f.time_spent.as_deref())
        .map(parse_time_spent_to_minutes)
        .unwrap_or(0);

    (description, time_spent)
}

fn parse_time_spent_to_minutes(s: &str) -> u64 {
    let s = s.trim().to_lowercase();
    let mut total: u64 = 0;

    if let Some(pos) = s.find('h') {
        if let Ok(hours) = s[..pos].trim().parse::<u64>() {
            total = total.wrapping_add(hours.wrapping_mul(60));
        }
    }

    if let Some(pos) = s.find('m') {
        let start = s.rfind('h').map(|p| p.wrapping_add(1)).unwrap_or(0);
        if let Ok(mins) = s[start..pos].trim().parse::<u64>() {
            total = total.wrapping_add(mins);
        }
    }

    total
}
