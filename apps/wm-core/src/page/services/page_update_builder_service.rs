use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::engine::{AcceptanceCriterion, EngineState, PageType};
use crate::error::{ToolError, ToolResult};
use crate::page_repo::{FsPageRepo, PageRepo};
use crate::shared::traits::Builder;

use crate::page::helpers::yaml_helper::{
    ac_set_checked, extract_yaml_string_value, remove_yaml_block, set_yaml_field,
};

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
}

pub fn update_page_with_repo(
    engine: &Arc<EngineState>,
    id: &str,
    updates: &PageUpdateParams,
    repo: &dyn PageRepo,
) -> ToolResult<()> {
    let snapshot = engine.graph.load();
    let index = &snapshot.1;
    let node_idx = index
        .get(id)
        .ok_or_else(|| ToolError::not_found("page", id))?;
    let meta = &snapshot.0[*node_idx];

    let file_path = &meta.path;
    if !repo.exists(file_path) {
        return Err(ToolError::not_found("page", id));
    }

    let content = repo.read_to_string(file_path)?;

    let (existing_fm, body) = crate::parser::extract_frontmatter(&content);

    let mut new_fm = existing_fm
        .as_ref()
        .map(crate::parser::frontmatter_to_yaml)
        .unwrap_or_default();

    if let Some(status) = updates.status.as_deref() {
        if meta.page_type == PageType::Task {
            let new_status = crate::parser::parse_page_status(status);
            let file_status_str = existing_fm.as_ref().and_then(|fm| fm.status.as_deref());
            let current_status = file_status_str
                .map(crate::parser::parse_page_status)
                .unwrap_or_else(|| meta.status.clone());
            if let Err(msg) = current_status.can_transition_to(&new_status) {
                return Err(ToolError::internal(msg));
            }
        }
        new_fm = set_yaml_field(&new_fm, "status", status);
    }

    if let Some(priority) = updates.priority.as_deref() {
        new_fm = set_yaml_field(&new_fm, "priority", priority);
    }

    if let Some(assignee) = updates.assignee.as_deref() {
        new_fm = set_yaml_field(&new_fm, "assignee", assignee);
    }

    if let Some(ref tag_list) = updates.tags {
        new_fm = remove_yaml_block(&new_fm, "tags");
        if !tag_list.is_empty() {
            new_fm.push_str(&format!("tags: [{}]\n", tag_list.join(", ")));
        }
    }

    if let Some(ref ac_list) = updates.acceptance_criteria {
        new_fm = remove_yaml_block(&new_fm, "acceptance_criteria");
        if !ac_list.is_empty() {
            new_fm.push_str("acceptance_criteria:\n");
            for ac in ac_list {
                new_fm.push_str(&format!(
                    "  - {{text: \"{}\", checked: {}}}\n",
                    ac.text, ac.checked
                ));
            }
        }
    }

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

    let full = format!("---\n{}---\n\n{}", new_fm, final_body);
    repo.write(file_path.as_path(), full.as_bytes())?;

    engine.notify_file_changed(file_path);

    // Incrementally update BM25: remove old sections, add new sections
    let page_id = meta.id.clone();
    crate::page::services::page_crud_service::update_bm25_for_page(
        engine,
        &page_id,
        &full,
        file_path,
        false,
    );

    Ok(())
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
