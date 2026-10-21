use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};

use crate::engine::{AcceptanceCriterion, EngineState, PageType, WikiPageMeta};
use crate::error::{ToolError, ToolResult};
use crate::page_repo::{FsPageRepo, PageRepo};
use crate::shared::traits::Builder;

use crate::page::helpers::{
    ac_set_checked, extract_yaml_string_value, remove_yaml_block, set_yaml_field,
    set_yaml_value_field,
};
use crate::page::services::page_create_builder_service::apply_extra_frontmatter;

const TAGS_KEY: &str = "tags";
const ACCEPTANCE_CRITERIA_KEY: &str = "acceptance_criteria";

static PAGE_WRITE_LOCKS: LazyLock<Mutex<HashMap<PathBuf, Arc<Mutex<()>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn page_write_lock(path: &Path) -> Arc<Mutex<()>> {
    let key = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let mut locks = PAGE_WRITE_LOCKS.lock().unwrap_or_else(|e| e.into_inner());
    locks.entry(key).or_default().clone()
}

#[derive(Default)]
pub struct PageUpdateParams {
    pub title: Option<String>,
    pub content: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub tags: Option<Vec<String>>,
    pub relates_to: Option<Vec<serde_json::Value>>,
    pub remove_relates_to: Option<String>,
    pub acceptance_criteria: Option<Vec<AcceptanceCriterion>>,
    pub implementation_plan: Option<String>,
    pub implementation_notes: Option<String>,
    pub append_notes: Option<String>,
    pub r#type: Option<String>,
    pub checked_ac: Option<Vec<u64>>,
    pub unchecked_ac: Option<Vec<u64>>,
    pub time_started: Option<String>,
    pub time_spent: Option<String>,
    pub extra_frontmatter: Option<serde_json::Map<String, serde_json::Value>>,
}

pub fn update_page_with_repo(
    engine: &Arc<EngineState>,
    id: &str,
    updates: &PageUpdateParams,
    repo: &dyn PageRepo,
) -> ToolResult<()> {
    let meta = crate::page::services::page_crud_service::resolve_page_meta(engine, id, repo)?;

    let file_path = &meta.path;
    if !repo.exists(file_path) {
        return Err(ToolError::not_found("page", id));
    }

    let full = with_page_write_lock(file_path, || {
        write_frontmatter_changes(file_path, &meta, updates, repo)
    })?;

    refresh_after_write(engine, file_path, &meta, &full);

    Ok(())
}

pub fn link_page(
    engine: &Arc<EngineState>,
    id: &str,
    edge_type: &str,
    target: &str,
) -> ToolResult<()> {
    let meta =
        crate::page::services::page_crud_service::resolve_page_meta(engine, id, &FsPageRepo)?;

    let file_path = &meta.path;
    if !FsPageRepo.exists(file_path) {
        return Err(ToolError::not_found("page", id));
    }

    let full = with_page_write_lock(file_path, || {
        let merged = merge_relates_to_from_file(file_path, edge_type, target)?;
        let updates = PageUpdateParams {
            relates_to: Some(merged),
            ..Default::default()
        };
        write_frontmatter_changes(file_path, &meta, &updates, &FsPageRepo)
    })?;

    refresh_after_write(engine, file_path, &meta, &full);

    Ok(())
}

fn with_page_write_lock<T>(path: &Path, f: impl FnOnce() -> ToolResult<T>) -> ToolResult<T> {
    let write_lock = page_write_lock(path);
    let _guard = write_lock.lock().unwrap_or_else(|e| e.into_inner());
    f()
}

fn write_frontmatter_changes(
    file_path: &Path,
    meta: &WikiPageMeta,
    updates: &PageUpdateParams,
    repo: &dyn PageRepo,
) -> ToolResult<String> {
    let content = repo.read_to_string(file_path)?;

    let (existing_fm, body) = wm_engine::parser::extract_frontmatter(&content);
    let (raw_fm_opt, _) = wm_engine::parser::extract_raw_frontmatter(&content);

    let mut new_fm = match raw_fm_opt {
        Some(raw) => raw,
        None => existing_fm
            .as_ref()
            .map(wm_engine::parser::frontmatter_to_yaml)
            .unwrap_or_default(),
    };

    if let Some(status) = updates.status.as_deref() {
        if meta.page_type == PageType::Task {
            let new_status = wm_engine::parser::parse_page_status(status);
            let file_status = existing_fm.as_ref().map(|fm| fm.status.clone());
            let current_status = file_status.unwrap_or_else(|| meta.status.clone());
            if let Err(msg) = current_status.can_transition_to(&new_status) {
                return Err(ToolError::internal(msg));
            }
        }
        new_fm = set_yaml_field(&new_fm, "status", status);
    }

    if let Some(title) = updates.title.as_deref() {
        new_fm = set_yaml_field(&new_fm, "title", title);
    }

    if let Some(r#type) = updates.r#type.as_deref() {
        new_fm = set_yaml_field(&new_fm, "type", r#type);
    }

    if let Some(priority) = updates.priority.as_deref() {
        new_fm = set_yaml_field(&new_fm, "priority", priority);
    }

    if let Some(assignee) = updates.assignee.as_deref() {
        new_fm = set_yaml_field(&new_fm, "assignee", assignee);
    }

    if let Some(ref ts) = updates.time_started {
        new_fm = set_yaml_field(&new_fm, "time_started", ts);
    }

    if let Some(ref tsp) = updates.time_spent {
        new_fm = set_yaml_field(&new_fm, "time_spent", tsp);
    }

    new_fm = apply_tags(&new_fm, updates.tags.as_deref());

    new_fm = apply_acceptance_criteria(&new_fm, updates.acceptance_criteria.as_deref());

    if let Some(ref plan) = updates.implementation_plan {
        new_fm = set_yaml_field(&new_fm, "implementation_plan", plan);
    }

    if let Some(ref notes) = updates.implementation_notes {
        new_fm = set_yaml_field(&new_fm, "implementation_notes", notes);
    }

    if let Some(ref append) = updates.append_notes {
        let existing = extract_yaml_string_value(&new_fm, "implementation_notes");
        let merged = if existing.is_empty() {
            append.to_string()
        } else {
            format!("{}\n{}", existing, append)
        };
        new_fm = set_yaml_field(&new_fm, "implementation_notes", &merged);
    }

    let final_body = if let Some(new_content) = updates.content.as_deref() {
        new_content
    } else {
        body
    };

    new_fm = apply_relates_to(new_fm, updates);
    new_fm = apply_checked_ac(new_fm, updates);

    if let Some(ref extra) = updates.extra_frontmatter {
        new_fm = apply_extra_frontmatter(&new_fm, Some(extra))?;
    }

    wm_engine::validate_required_keys(&new_fm).map_err(ToolError::invalid_params)?;

    let full = format!("---\n{}---\n\n{}", new_fm, final_body);
    repo.write(file_path, full.as_bytes())?;

    Ok(full)
}

fn apply_tags(frontmatter: &str, tags: Option<&[String]>) -> String {
    let Some(tags) = tags else {
        return frontmatter.to_owned();
    };
    set_yaml_value_field(frontmatter, TAGS_KEY, &serde_json::json!(tags))
}

fn apply_acceptance_criteria(
    frontmatter: &str,
    criteria: Option<&[AcceptanceCriterion]>,
) -> String {
    let Some(criteria) = criteria else {
        return frontmatter.to_owned();
    };
    set_yaml_value_field(
        frontmatter,
        ACCEPTANCE_CRITERIA_KEY,
        &serde_json::json!(criteria),
    )
}

fn refresh_after_write(
    engine: &Arc<EngineState>,
    file_path: &Path,
    meta: &WikiPageMeta,
    full: &str,
) {
    let wiki_dir = crate::page::services::page_crud_service::wiki_dir_for(engine);
    let anchored = crate::page::services::page_crud_service::anchored_page_path(engine, file_path);
    crate::graph::handle_file_change(&wiki_dir, &anchored, engine);

    engine.notify_file_changed(file_path);

    crate::page::services::page_crud_service::update_bm25_for_page(
        engine, &meta.id, full, file_path, false,
    );
}

fn merge_relates_to_from_file(
    file_path: &Path,
    edge_type: &str,
    target: &str,
) -> ToolResult<Vec<serde_json::Value>> {
    let content = FsPageRepo.read_to_string(file_path)?;
    let (fm, _body) = wm_engine::parser::extract_frontmatter(&content);
    let mut edges: Vec<serde_json::Value> = fm
        .map(|f| f.relates_to)
        .unwrap_or_default()
        .into_iter()
        .map(|(edge_type, target)| {
            serde_json::json!({ "type": edge_type.as_yaml_str(), "target": target })
        })
        .collect();
    if edges
        .iter()
        .any(|e| edge_matches_json(e, edge_type, target))
    {
        return Ok(edges);
    }
    edges.push(serde_json::json!({ "type": edge_type, "target": target }));
    Ok(edges)
}

fn edge_matches_json(edge: &serde_json::Value, edge_type: &str, target: &str) -> bool {
    edge.get("type").and_then(|v| v.as_str()) == Some(edge_type)
        && edge.get("target").and_then(|v| v.as_str()) == Some(target)
}

fn apply_relates_to(mut new_fm: String, updates: &PageUpdateParams) -> String {
    if let Some(ref rel_list) = updates.relates_to {
        new_fm = remove_yaml_block(&new_fm, "relates_to");
        if !rel_list.is_empty() {
            new_fm.push_str("relates_to:\n");
            for rel in rel_list {
                let edge_type = rel
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("relates_to");
                let target = rel.get("target").and_then(|v| v.as_str()).unwrap_or("");
                new_fm.push_str(&format!(
                    "  - {{type: {}, target: {}}}\n",
                    edge_type, target
                ));
            }
        }
    }

    if let Some(remove_target) = updates.remove_relates_to.as_deref() {
        let mut kept: Vec<String> = Vec::new();
        for line in new_fm.lines() {
            if line.trim().starts_with("- {") && line.contains(remove_target) {
                continue;
            }
            kept.push(line.to_string());
        }
        new_fm = remove_yaml_block(&new_fm, "relates_to");
        let has_relates = kept.iter().any(|l| l.trim().starts_with("- {"));
        if has_relates {
            new_fm.push_str("relates_to:\n");
            for k in kept {
                if k.trim().starts_with("- {") {
                    new_fm.push_str(&format!("  {}\n", k.trim_start()));
                }
            }
        }
    }

    new_fm
}

fn apply_checked_ac(mut new_fm: String, updates: &PageUpdateParams) -> String {
    if let Some(ref check_list) = updates.checked_ac {
        for &idx in check_list.iter() {
            let i = usize::try_from(idx).unwrap_or(usize::MAX);
            new_fm = ac_set_checked(&new_fm, i, true);
        }
    }
    if let Some(ref uncheck_list) = updates.unchecked_ac {
        for &idx in uncheck_list.iter() {
            let i = usize::try_from(idx).unwrap_or(usize::MAX);
            new_fm = ac_set_checked(&new_fm, i, false);
        }
    }
    new_fm
}

impl Builder<Self> for PageUpdateParams {
    fn build(self) -> Result<Self, crate::error::ToolError> {
        Ok(self)
    }
}

pub fn update_page(
    engine: &Arc<EngineState>,
    id: &str,
    updates: &PageUpdateParams,
) -> ToolResult<()> {
    update_page_with_repo(engine, id, updates, &FsPageRepo)
}
