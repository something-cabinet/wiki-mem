use crate::engine::{MemoryEntry, MemoryStatus, PageType};
use crate::mcp::prelude::*;
use dashmap::DashMap;
use std::path::PathBuf;
use wm_constants::*;

use crate::page;
use crate::parser;

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
    store: &DashMap<String, MemoryEntry>,
    filter_tag: Option<&str>,
    filter_status: Option<&MemoryStatus>,
    limit: usize,
) -> Vec<serde_json::Value> {
    let mut entries: Vec<serde_json::Value> = Vec::new();
    for item in store.iter() {
        if entries.len() >= limit {
            break;
        }
        let mem = item.value();
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

fn evict_lowest_fsrs(store: &DashMap<String, MemoryEntry>, stability_days: f64) {
    let now = chrono::Utc::now();
    let mut lowest_score = f64::MAX;
    let mut lowest_key = String::new();

    for item in store.iter() {
        let mem = item.value();
        let updated = chrono::DateTime::parse_from_rfc3339(&mem.updated_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or(now);
        let days = f64::from((now - updated).num_seconds() as i32) / 86400.0;
        let score =
            wm_search::recency_boost(days, &crate::config::RecencyModel::Fsrs, stability_days);
        if score < lowest_score {
            lowest_score = score;
            lowest_key = mem.id.clone();
        }
    }

    if !lowest_key.is_empty() {
        store.remove(&lowest_key);
    }
}

fn slugify(text: &str) -> String {
    let slug: String = text
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();

    let mut result = String::with_capacity(slug.len());
    let mut prev_hyphen = false;
    for c in slug.chars() {
        if c == '-' {
            if !prev_hyphen {
                result.push('-');
                prev_hyphen = true;
            }
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }
    result.trim_matches('-').to_string()
}

fn parse_memory_status(s: &str) -> Option<MemoryStatus> {
    match s {
        "active" => Some(MemoryStatus::Active),
        "stale" => Some(MemoryStatus::Stale),
        "archived" => Some(MemoryStatus::Archived),
        _ => None,
    }
}

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_typed(
        "wm_memory",
        "Manage memory entries (list, get, add, update, delete, promote)",
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
                                    "content": body.trim(),
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
                        "title": fm.as_ref().and_then(|f| f.title.as_deref()).unwrap_or(&id),
                        "content": body.trim(),
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
                    let slug = slugify(&title);
                    let tags = tags.unwrap_or_default();
                    let layer = layer.unwrap_or_else(|| "project".into());

                    if is_session(&layer) {
                        let (capacity, stability_days) = e
                            .config
                            .read()
                            .ok()
                            .map(|cfg| {
                                let cap: usize = cfg
                                    .runtime_memory_max_entries
                                    .unwrap_or(DEFAULT_MEMORY_CAPACITY as u32)
                                    as usize;
                                // u32 cast to usize is safe on 64-bit
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
                        if e.session_memory.len() >= capacity {
                            evict_lowest_fsrs(&e.session_memory, stability_days);
                        }
                        e.session_memory.insert(id.clone(), mem);
                        return Ok(serde_json::json!({
                            "id": id,
                            "status": "created",
                            "layer": "session",
                        }));
                    }

                    let path = format!("memory/{}", slug);
                    let tags_str = if tags.is_empty() {
                        String::new()
                    } else {
                        format!("tags: [{}]\n", tags.join(", "))
                    };
                    let frontmatter = format!(
                        "title: {}\ntype: memory\n{}status: active\n",
                        title, tags_str
                    );

                    let id = page::create_page(&e, &path, &frontmatter, &content)?;

                    Ok(serde_json::json!({
                        "id": id,
                        "title": title,
                        "content": content,
                        "tags": tags,
                        "status": "created",
                        "layer": layer,
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
                    let raw = page::get_page_raw(&e, &id)?;

                    let home = std::env::var("HOME")
                        .or_else(|_| std::env::var("USERPROFILE"))
                        .unwrap_or_else(|_| ".".into());
                    let global_dir = PathBuf::from(home)
                        .join(WM_DIR)
                        .join(WIKI_DIR)
                        .join("memory");
                    std::fs::create_dir_all(&global_dir).map_err(|e| {
                        ToolError::io_error("create_dir", global_dir.to_string_lossy(), e)
                    })?;

                    let path_part = id.replace(':', "/");
                    let path_part = path_part.strip_prefix("wiki/").unwrap_or(&path_part);
                    let global_path = global_dir.join(format!("{}.md", path_part));

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
