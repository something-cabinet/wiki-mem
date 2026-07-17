use std::sync::Arc;

use crate::engine::{EngineState, PageStatus, PageType};
use crate::error::ToolError;
use crate::mcp::transport::ToolRegistry;

use crate::page;
use crate::version::{FieldChange, VersionStore};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ─── Action enum ────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
enum WmPageAction {
    #[schemars(description = "List all wiki pages")]
    List {
        r#type: Option<String>,
        limit: Option<usize>,
    },
    #[schemars(description = "Get page content by ID")]
    Get { id: String },
    #[schemars(description = "Create a new wiki page")]
    Create {
        path: String,
        title: String,
        content: Option<String>,
        r#type: Option<String>,
        tags: Option<Vec<String>>,
        status: Option<String>,
    },
    #[schemars(description = "Update page frontmatter fields")]
    Update {
        id: String,
        title: Option<String>,
        content: Option<String>,
        status: Option<String>,
        tags: Option<Vec<String>>,
        r#type: Option<String>,
        relates_to: Option<Vec<serde_json::Value>>,
        notes: Option<String>,
        append_notes: Option<String>,
    },
    #[schemars(description = "Delete a page by ID")]
    Delete { id: String },
    #[schemars(description = "Add a typed edge between pages")]
    Link {
        id: String,
        target: String,
        edge_type: Option<String>,
    },
    #[schemars(description = "Remove a typed edge between pages")]
    Unlink { id: String, target: String },
}

// ─── Output types ───────────────────────────────────────────

#[derive(Serialize)]
struct WmPageGetOutput {
    id: String,
    content: String,
    sections: Vec<PageSectionOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    updated_at: Option<String>,
}

#[derive(Serialize)]
struct PageSectionOutput {
    header: String,
    body: String,
}

#[derive(Serialize)]
struct WmPageCreateOutput {
    id: String,
    path: String,
    r#type: String,
}

#[derive(Serialize)]
struct WmPageListOutput {
    pages: Vec<serde_json::Value>,
    total: usize,
}

#[derive(Serialize)]
struct WmPageUpdateOutput {
    id: String,
    status: String,
}

#[derive(Serialize)]
struct WmPageDeleteOutput {
    id: String,
    status: String,
}

#[derive(Serialize)]
struct WmPageLinkOutput {
    id: String,
    target: String,
    r#type: String,
    status: String,
}

#[derive(Serialize)]
struct WmPageUnlinkOutput {
    id: String,
    target: String,
    status: String,
}

// ─── Tool Registration ──────────────────────────────────────

/// Register the single wm_doc tool (consolidated page CRUD + typed pages + edges)
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    registry.register_typed(
        "wm_page",
        "Page CRUD operations: list, get, create, update, delete, link, unlink",
        move |input: WmPageAction| -> Result<serde_json::Value, ToolError> {
            match input {
                WmPageAction::List { r#type, limit: _ } => {
                    let page_type_filter = r#type.as_deref().and_then(|t| {
                        Some(match t {
                            "task" => PageType::Task,
                            "spec" => PageType::Spec,
                            "concept" => PageType::Concept,
                            "pattern" => PageType::Pattern,
                            "decision" => PageType::Decision,
                            "memory" => PageType::Memory,
                            "howto" => PageType::Howto,
                            "reference" => PageType::Reference,
                            "note" => PageType::Note,
                            "rule" => PageType::Rule,
                            _ => return None,
                        })
                    });
                    let pages = page::list_pages(&engine, page_type_filter.as_ref())?;
                    let total = pages.len();
                    Ok(serde_json::to_value(WmPageListOutput { pages, total })
                        .unwrap_or(serde_json::Value::Null))
                }
                WmPageAction::Get { id } => {
                    // Try graph-backed lookup first
                    let content_result = page::get_page(&engine, &id);
                    let content = match content_result {
                        Ok(c) => c,
                        Err(_) => {
                            // Fallback: read directly from filesystem (for pages not in graph)
                            let root = engine.project_root.read()
                                .map(|r| r.clone())
                                .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
                            let path_part = id.replace(':', "/");
                            let path_part = path_part.strip_prefix("wiki/").unwrap_or(&path_part);
                            let file_path = root.join(".wm").join("wiki").join(format!("{}.md", path_part));
                            if !file_path.exists() {
                                return Err(ToolError::not_found("page", &id));
                            }
                            let raw = std::fs::read_to_string(&file_path)
                                .map_err(|_| ToolError::not_found("page", &id))?;
                            let sections = crate::parser::split_sections(&raw);
                            crate::engine::WikiPageContent {
                                raw,
                                sections: sections
                                    .into_iter()
                                    .map(|(header, body)| {
                                        let section_id = format!("{}#{}", id, header.to_lowercase().replace(' ', "-"));
                                        crate::engine::SectionDoc {
                                            section_id,
                                            page_id: id.clone(),
                                            header,
                                            body,
                                        }
                                    })
                                    .collect(),
                                meta: None,
                            }
                        }
                    };
                    let (tags, page_type, created_at, updated_at) = content
                        .meta
                        .as_ref()
                        .map(|m| {
                            let tags = if m.tags.is_empty() {
                                None
                            } else {
                                Some(m.tags.clone())
                            };
                            (tags, Some(m.page_type.as_str().to_string()), Some(m.created_at.clone()), Some(m.updated_at.clone()))
                        })
                        .unwrap_or((None, None, None, None));
                    Ok(serde_json::to_value(WmPageGetOutput {
                        id,
                        content: content.raw,
                        sections: content
                            .sections
                            .iter()
                            .map(|s| PageSectionOutput {
                                header: s.header.clone(),
                                body: s.body.clone(),
                            })
                            .collect(),
                        tags,
                        r#type: page_type,
                        description: None,
                        created_at,
                        updated_at,
                    })
                    .unwrap_or(serde_json::Value::Null))
                }
                WmPageAction::Create {
                    path,
                    title,
                    content,
                    r#type,
                    tags,
                    status,
                } => {
                    let content = content.unwrap_or_default();
                    let page_type = if let Some(ref t) = r#type {
                        serde_json::from_value(serde_json::Value::String(t.clone()))
                            .map_err(|e| {
                                ToolError::invalid_params(format!(
                                    "Invalid page type '{}': {}",
                                    t, e
                                ))
                            })?
                    } else {
                        let first_segment = path
                            .trim_start_matches("wiki/")
                            .split('/')
                            .next()
                            .unwrap_or("concept");
                        match first_segment {
                            "tasks" => PageType::Task,
                            "specs" => PageType::Spec,
                            "concepts" => PageType::Concept,
                            "patterns" => PageType::Pattern,
                            "decisions" => PageType::Decision,
                            "howto" => PageType::Howto,
                            "memory" => PageType::Memory,
                            "reference" => PageType::Reference,
                            "notes" => PageType::Note,
                            "rules" => PageType::Rule,
                            _ => PageType::Concept,
                        }
                    };
                    let page_type_str = page_type.as_str();

                    // Parse and validate status if provided
                    let parsed_status = status
                        .as_deref()
                        .map(|s| {
                            serde_json::from_value::<PageStatus>(serde_json::Value::String(
                                s.to_string(),
                            ))
                            .map_err(|e| {
                                ToolError::invalid_params(format!("Invalid status '{}': {}", s, e))
                            })
                        })
                        .transpose()?;

                    if let Some(ref ps) = parsed_status {
                        if !page_type.allowed_statuses().contains(ps) {
                            return Err(ToolError::invalid_params(format!(
                                "Invalid status '{}' for '{}' page. Allowed: {}",
                                ps.as_str(),
                                page_type_str,
                                page_type
                                    .allowed_statuses()
                                    .iter()
                                    .map(|s| s.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )));
                        }
                    }

                    let mut frontmatter =
                        format!("title: {}\ntype: {}\n", title, page_type_str);
                    if let Some(ref ps) = parsed_status {
                        frontmatter.push_str(&format!("status: {}\n", ps.as_str()));
                    }
                    if let Some(ref t) = tags {
                        let tags_str = t.join(", ");
                        frontmatter.push_str(&format!("tags: [{}]\n", tags_str));
                    }
                    let id =
                        page::create_page(&engine, &path, &frontmatter, &content)?;

                    // Rebuild graph immediately so subsequent tool calls find the page
                    let root = engine.project_root.read()
                        .map(|r| r.clone())
                        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
                    let wiki_dir = root.join(".wm").join("wiki");
                    if wiki_dir.exists() {
                        let custom_types: Vec<String> = engine.config.read()
                            .ok()
                            .map(|cfg| cfg.custom_edge_types.clone())
                            .unwrap_or_default();
                        crate::graph::rebuild_graph_snapshot(&engine.graph, &wiki_dir, &custom_types);
                    }

                    let e2 = engine.clone();
                    engine.index_scheduler.submit("page", move || {
                        let root = e2
                            .project_root
                            .read()
                            .map(|r| r.clone())
                            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
                        let wiki_dir = root.join(".wm").join("wiki");
                        let sections =
                            crate::graph::build_sections_from_wiki(&wiki_dir);
                        let docs: Vec<crate::search::IndexedDoc> = sections
                            .iter()
                            .map(|s| crate::search::IndexedDoc {
                                id: s.section_id.clone(),
                                fields: vec![
                                    crate::search::Field::new("header", &s.header, 4.0),
                                    crate::search::Field::new("body", &s.body, 1.0),
                                ],
                            })
                            .collect();
                        e2.bm25_index
                            .store(Arc::new(crate::search::Bm25Index::build(docs)));
                    });
                    Ok(serde_json::to_value(WmPageCreateOutput {
                        id,
                        path,
                        r#type: page_type_str.to_string(),
                    })
                    .unwrap_or(serde_json::Value::Null))
                }
                WmPageAction::Update {
                    id,
                    title,
                    content,
                    status,
                    tags,
                    r#type,
                    relates_to,
                    notes,
                    append_notes,
                } => {
                    // Validate status against page type's allowed statuses
                    if let Some(ref status_str) = status {
                        let snapshot = engine.graph.load();
                        let index = &snapshot.1;
                        if let Some(node_idx) = index.get(&id) {
                            let meta = &snapshot.0[*node_idx];
                            let parsed_status: PageStatus = serde_json::from_value(
                                serde_json::Value::String(status_str.clone()),
                            )
                            .map_err(|e| {
                                ToolError::invalid_params(format!(
                                    "Invalid status '{}': {}",
                                    status_str, e
                                ))
                            })?;
                            if !meta.page_type.allowed_statuses().contains(&parsed_status) {
                                return Err(ToolError::invalid_params(format!(
                                    "Invalid status '{}' for '{}' page. Allowed: {}",
                                    parsed_status.as_str(),
                                    meta.page_type.as_str(),
                                    meta.page_type
                                        .allowed_statuses()
                                        .iter()
                                        .map(|s| s.as_str())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                )));
                            }
                        }
                    }

                    // ── Version tracking ─────────────────────────────────
                    let root = engine.project_root.read()
                        .map(|r| r.clone())
                        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
                    let store = VersionStore::new(root.join(".wm"));

                    // Get page metadata for old values
                    let snapshot = engine.graph.load();
                    let index = &snapshot.1;
                    let page_meta = index.get(&id).map(|idx| &snapshot.0[*idx]);

                    let file_path = page_meta.map(|m| &m.path);
                    let old_content = file_path
                        .and_then(|p| std::fs::read_to_string(p).ok())
                        .unwrap_or_default();
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
                    if tags.is_some() {
                        let old_val = old_fm.as_ref().map(|fm| fm.tags.clone());
                        changes.push(FieldChange {
                            field: "tags".into(),
                            old_value: old_val.map(|v| serde_json::to_value(v).unwrap_or_default()),
                            new_value: tags.clone().map(|v| serde_json::to_value(v).unwrap_or_default()),
                        });
                    }
                    if content.is_some() {
                        let old_val = Some(_old_body.trim().to_string());
                        changes.push(FieldChange {
                            field: "content".into(),
                            old_value: old_val.map(serde_json::Value::String),
                            new_value: content.clone().map(serde_json::Value::String),
                        });
                    }
                    if r#type.is_some() {
                        let old_val = old_fm.as_ref().and_then(|fm| fm.page_type.as_deref());
                        changes.push(FieldChange {
                            field: "type".into(),
                            old_value: old_val.map(|s| serde_json::Value::String(s.to_string())),
                            new_value: r#type.clone().map(serde_json::Value::String),
                        });
                    }
                    if relates_to.is_some() {
                        changes.push(FieldChange {
                            field: "relates_to".into(),
                            old_value: None,
                            new_value: relates_to.clone().map(|v| serde_json::to_value(v).unwrap_or_default()),
                        });
                    }

                    // Save as doc version
                    let doc_path = file_path
                        .and_then(|p| p.strip_prefix(&root).ok())
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();
                    store.save_doc_version(&id, &doc_path, changes)?;
                    // ── End version tracking ─────────────────────────────

                    let params = page::PageUpdateParams {
                        title,
                        content,
                        status,
                        tags,
                        relates_to,
                        r#type,
                        implementation_notes: notes,
                        append_notes,
                        ..Default::default()
                    };
                    page::update_page(&engine, &id, &params)?;
                    Ok(serde_json::to_value(WmPageUpdateOutput {
                        id,
                        status: "updated".into(),
                    })
                    .unwrap_or(serde_json::Value::Null))
                }
                WmPageAction::Delete { id } => {
                    page::delete_page(&engine, &id)?;
                    Ok(serde_json::to_value(WmPageDeleteOutput {
                        id,
                        status: "deleted".into(),
                    })
                    .unwrap_or(serde_json::Value::Null))
                }
                WmPageAction::Link {
                    id,
                    target,
                    edge_type,
                } => {
                    let edge_type = edge_type.unwrap_or_else(|| "relates_to".into());
                    let params = page::PageUpdateParams {
                        relates_to: Some(vec![serde_json::json!({
                            "type": edge_type,
                            "target": target.clone()
                        })]),
                        ..Default::default()
                    };
                    page::update_page(&engine, &id, &params)?;
                    Ok(serde_json::to_value(WmPageLinkOutput {
                        id,
                        target,
                        r#type: edge_type,
                        status: "linked".into(),
                    })
                    .unwrap_or(serde_json::Value::Null))
                }
                WmPageAction::Unlink { id, target } => {
                    let params = page::PageUpdateParams {
                        remove_relates_to: Some(target.clone()),
                        ..Default::default()
                    };
                    page::update_page(&engine, &id, &params)?;
                    Ok(serde_json::to_value(WmPageUnlinkOutput {
                        id,
                        target,
                        status: "unlinked".into(),
                    })
                    .unwrap_or(serde_json::Value::Null))
                }
            }
        },
    );
}
