use crate::prelude::*;
use std::path::{Path, PathBuf};
use wm_constants::*;
use wm_core::engine::{MemoryEntry, MemoryState, MemoryStatus, PageType};

use wm_core::engine::PageStatus;
use wm_core::page;
use wm_core::page::helpers::FrontmatterValue;
use wm_engine::parser;

#[derive(Deserialize, JsonSchema)]
struct WmMemoryAddSchema {
    #[serde(rename = "category")]
    #[schemars(description = "Category")]
    _category: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
enum WmMemoryAction {
    List {
        #[schemars(description = "Memory layer: project/global/session")]
        layer: Option<String>,
        #[schemars(description = "Filter by memory status: active/stale/archived")]
        status: Option<String>,
        #[schemars(description = "Filter by tag")]
        tag: Option<String>,
    },
    Get {
        #[schemars(description = "Memory entry ID (wiki page ID, e.g. 'wiki:memory:my-title')")]
        id: String,
    },
    Add {
        #[schemars(description = "Title")]
        title: String,
        #[schemars(description = "Content")]
        content: String,
        #[schemars(description = "Tags")]
        tags: Option<Vec<String>>,
        #[schemars(description = "Memory layer: project/global/session")]
        layer: Option<String>,
        #[serde(flatten)]
        _schema: WmMemoryAddSchema,
    },
    Update {
        #[schemars(description = "Memory entry ID (wiki page ID)")]
        id: String,
        #[schemars(description = "New title")]
        title: Option<String>,
        #[schemars(description = "New content")]
        content: Option<String>,
        #[schemars(description = "New tags")]
        tags: Option<Vec<String>>,
    },
    Delete {
        #[schemars(description = "Memory entry ID to delete")]
        id: String,
    },
    Promote {
        #[schemars(description = "Memory entry ID to promote (wiki page ID)")]
        id: String,
    },
}

fn is_session(layer: &str) -> bool {
    layer == "session"
}

fn session_entries(
    store: &MemoryState,
    filter_tag: Option<&str>,
    filter_status: Option<&MemoryStatus>,
    limit: usize,
) -> Vec<serde_json::Value> {
    let mut entries: Vec<serde_json::Value> = Vec::new();
    for mem in store.snapshot() {
        if entries.len() >= limit {
            break;
        }
        if let Some(tag) = filter_tag {
            let has_tag = mem.tags.iter().any(|t| t.to_lowercase() == tag);
            if !has_tag {
                continue;
            }
        }
        if let Some(status) = filter_status {
            if mem.status.as_ref() != Some(status) {
                continue;
            }
        }
        entries.push(serde_json::json!({
            "id": mem.id,
            "title": mem.title,
            "content": mem.content,
            "tags": mem.tags,
            "status": mem.status,
            "createdAt": mem.created_at,
            "updatedAt": mem.updated_at,
        }));
    }
    entries
}

fn evict_lowest_fsrs(store: &MemoryState, stability_days: f64) {
    let now = chrono::Utc::now();
    let mut lowest_score = f64::MAX;
    let mut lowest_key = String::new();

    for mem in store.snapshot() {
        let updated = chrono::DateTime::parse_from_rfc3339(&mem.updated_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or(now);
        let days = f64::from((now - updated).num_seconds() as i32) / 86400.0;
        let score =
            wm_search::recency_boost(days, &wm_core::config::RecencyModel::Fsrs, stability_days);
        if score < lowest_score {
            lowest_score = score;
            lowest_key = mem.id.clone();
        }
    }

    if !lowest_key.is_empty() {
        store.remove(&lowest_key);
    }
}

fn parse_memory_status(s: &str) -> Option<MemoryStatus> {
    match s {
        "active" => Some(MemoryStatus::Active),
        "stale" => Some(MemoryStatus::Stale),
        "archived" => Some(MemoryStatus::Archived),
        _ => None,
    }
}

fn read_memory_from_disk_bypassing_cache(
    engine: &EngineState,
    id: &str,
) -> Result<(PathBuf, String), ToolError> {
    let page_id = id.split('#').next().unwrap_or(id);
    let root = engine
        .project_root
        .read()
        .map(|r| r.clone())
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    let rel = wm_core::page::resolve_page_path("", page_id)?;
    let project_path = if rel.is_absolute() {
        rel
    } else {
        root.join(rel)
    };
    if !project_path.exists() {
        return Err(ToolError::not_found("memory", id));
    }
    let raw = std::fs::read_to_string(&project_path)
        .map_err(|err| ToolError::io_error("read", project_path.to_string_lossy(), err))?;
    Ok((project_path, raw))
}

fn resolve_global_memory_dir() -> Result<PathBuf, ToolError> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    let global_dir = PathBuf::from(home)
        .join(WM_DIR)
        .join(WIKI_DIR)
        .join("memory");
    std::fs::create_dir_all(&global_dir)
        .map_err(|e| ToolError::io_error("create_dir", global_dir.to_string_lossy(), e))?;
    Ok(global_dir)
}

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_typed(
        "wm_memory",
        "Manage memory entries (list, get, add, update, delete, promote). add stores a record-backed memory page, or returns status 'skipped' with nearest evidence when a near-duplicate or an existing page blocks the write",
        move |input: WmMemoryAction| {
            match input {
                WmMemoryAction::List { layer, status, tag } => {
                    let filter_tag = tag.map(|s| s.to_lowercase());
                    let limit = 50usize;
                    let layer = layer.unwrap_or_else(|| "project".into());
                    let _filter_status = status.as_deref().and_then(parse_memory_status);

                    if is_session(&layer) {
                        let entries = session_entries(
                            &e.session_memory,
                            filter_tag.as_deref(),
                            _filter_status.as_ref(),
                            limit,
                        );
                        return Ok(serde_json::json!({
                            "entries": entries,
                            "total": entries.len(),
                        }));
                    }

                    let pages = page::list_pages(&e, Some(&PageType::Memory))?;
                    let mut entries: Vec<serde_json::Value> = Vec::new();

                    for p in &pages {
                        if entries.len() >= limit {
                            break;
                        }
                        let id = p["id"].as_str().unwrap_or("").to_string();
                        let title = p["title"].as_str().unwrap_or("").to_string();

                        if filter_tag.is_some() {
                            if let Ok(raw) = page::get_page_raw(&e, &id) {
                                let (fm, body) = parser::extract_frontmatter(&raw);
                                if let Some(ref tag) = filter_tag {
                                    let has_tag = fm
                                        .as_ref()
                                        .map(|f| f.tags.iter().any(|t| t.to_lowercase() == *tag))
                                        .unwrap_or(false);
                                    if !has_tag {
                                        continue;
                                    }
                                }
                                entries.push(serde_json::json!({
                                    "id": id,
                                    "title": title,
                                    "content": parser::readable_body(body),
                                    "tags": fm.as_ref().map(|f| f.tags.clone()).unwrap_or_default(),
                                    "createdAt": "",
                                    "updatedAt": "",
                                }));
                            }
                        } else {
                            entries.push(serde_json::json!({
                                "id": id,
                                "title": title,
                                "content": "",
                                "tags": [],
                                "createdAt": "",
                                "updatedAt": "",
                            }));
                        }
                    }

                    Ok(serde_json::json!({
                        "entries": entries,
                        "total": entries.len(),
                    }))
                }

                WmMemoryAction::Get { id } => {
                    let raw = page::get_page_raw(&e, &id)?;
                    let (fm, body) = parser::extract_frontmatter(&raw);
                    let tags = fm.as_ref().map(|f| f.tags.clone()).unwrap_or_default();

                    Ok(serde_json::json!({
                        "id": id,
                        "title": fm.as_ref().map(|f| f.title.as_str()).filter(|t| !t.is_empty()).unwrap_or(&id),
                        "content": parser::readable_body(body),
                        "tags": tags,
                        "createdAt": "",
                        "updatedAt": "",
                    }))
                }

                WmMemoryAction::Add {
                    title,
                    content,
                    tags,
                    layer,
                    ..
                } => {
                    let tags = tags.unwrap_or_default();
                    let layer = layer.unwrap_or_else(|| "project".into());

                    if is_session(&layer) {
                        let slug = wm_core::memory_gate::memory_slug(&title);
                        let (capacity, stability_days) = e
                            .config
                            .read()
                            .ok()
                            .map(|cfg| {
                                let cap: usize = cfg
                                    .runtime_memory_max_entries
                                    .unwrap_or(DEFAULT_MEMORY_CAPACITY as u32)
                                    as usize;
                                let stab = f64::from(cfg.search.scoring.recency_stability_days);
                                (cap, stab)
                            })
                            .unwrap_or((DEFAULT_MEMORY_CAPACITY, DEFAULT_MEMORY_STABILITY_DAYS));

                        let id = slug.clone();
                        let now = iso_now();
                        let mem = MemoryEntry {
                            id: id.clone(),
                            title,
                            content,
                            tags,
                            created_at: now.clone(),
                            updated_at: now,
                            status: None,
                        };
                        if e.session_memory.count() >= capacity {
                            evict_lowest_fsrs(&e.session_memory, stability_days);
                        }
                        e.session_memory.insert(mem);
                        return Ok(serde_json::json!({
                            "id": id,
                            "status": "created",
                            "layer": "session",
                        }));
                    }

                    let slug = wm_core::memory_gate::memory_slug(&title);
                    let decision = wm_core::memory_gate::evaluate_memory_write(
                        &e,
                        &title,
                        &content,
                        Some(layer.as_str()),
                    )?;

                    if !decision.store {
                        let reason = decision
                            .skip_reason
                            .map(|reason| reason.as_str())
                            .unwrap_or_default();
                        return Ok(serde_json::json!({
                            "id": format!("memory/{slug}"),
                            "status": "skipped",
                            "reason": reason,
                            "layer": layer,
                            "nearest": {
                                "id": decision.evidence.nearest_id,
                                "score": decision.evidence.nearest_score,
                            },
                        }));
                    }

                    let path = format!("memory/{slug}");
                    let mut fields: Vec<(&'static str, FrontmatterValue)> = vec![
                        ("title", FrontmatterValue::Scalar(title.clone())),
                        (
                            "type",
                            FrontmatterValue::Scalar(PageType::Memory.as_str().to_owned()),
                        ),
                    ];
                    if !tags.is_empty() {
                        fields.push(("tags", FrontmatterValue::List(tags.clone())));
                    }
                    fields.push((
                        "status",
                        FrontmatterValue::Scalar(PageStatus::Active.as_str().to_owned()),
                    ));

                    let body = decision.answers_body(&content)?;
                    let created = page::create_page_through_builder(
                        &e,
                        page::PageCreateRequest {
                            path,
                            page_type: PageType::Memory,
                            fields,
                            content: body,
                            extra_frontmatter: None,
                        },
                    )?;
                    let id = created.id;

                    Ok(serde_json::json!({
                        "id": id,
                        "title": title,
                        "content": content,
                        "tags": tags,
                        "status": "created",
                        "layer": layer,
                        "nearest": {
                            "id": decision.evidence.nearest_id,
                            "score": decision.evidence.nearest_score,
                        },
                    }))
                }

                WmMemoryAction::Update {
                    id,
                    title,
                    content,
                    tags,
                } => {
                    let params = page::PageUpdateParams {
                        title,
                        content,
                        tags,
                        ..Default::default()
                    };
                    page::update_page(&e, &id, &params)?;

                    Ok(serde_json::json!({
                        "id": id,
                        "status": "updated",
                    }))
                }

                WmMemoryAction::Delete { id } => {
                    page::delete_page(&e, &id)?;
                    Ok(serde_json::json!({
                        "id": id,
                        "status": "deleted"
                    }))
                }

                WmMemoryAction::Promote { id } => {
                    let (project_path, raw) =
                        read_memory_from_disk_bypassing_cache(&e, &id)?;

                    let global_dir = resolve_global_memory_dir()?;
                    let global_path = global_memory_path(&global_dir, &project_path);

                    std::fs::write(&global_path, raw.as_bytes()).map_err(|e| {
                        ToolError::io_error("write", global_path.to_string_lossy(), e)
                    })?;

                    Ok(serde_json::json!({
                        "id": id,
                        "status": "promoted",
                        "source": "project",
                        "target": "global"
                    }))
                }
            }
        },
    );
}

fn iso_now() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn global_memory_path(global_dir: &Path, project_path: &Path) -> PathBuf {
    global_dir.join(
        project_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("memory.md"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_memory_path_appends_only_file_name() {
        let global_dir = Path::new("/home/u/.wm/wiki/memory");
        let project_path = Path::new("/proj/.wm/wiki/memory/my-title.md");
        assert_eq!(
            global_memory_path(global_dir, project_path),
            PathBuf::from("/home/u/.wm/wiki/memory/my-title.md")
        );
    }

    #[test]
    fn global_memory_path_never_double_appends_memory_segment() {
        let global_dir = Path::new("/home/u/.wm/wiki/memory");
        let project_path = Path::new("/proj/.wm/wiki/memory/my-title.md");
        let target = global_memory_path(global_dir, project_path);
        let rel = target.strip_prefix(global_dir).expect("under global dir");
        assert!(
            !rel.to_string_lossy().contains("memory/memory"),
            "target must not double-append memory/: {}",
            target.display()
        );
    }

    #[test]
    fn is_session_only_matches_session_layer() {
        assert!(is_session("session"));
        assert!(!is_session("project"));
        assert!(!is_session("global"));
        assert!(!is_session("n"));
    }
}
