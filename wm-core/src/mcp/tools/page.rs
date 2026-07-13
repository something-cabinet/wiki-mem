use std::sync::Arc;

use serde_json::json;

use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::transport::ToolRegistry;
use crate::mcp::typed::TypedRegister;
use crate::page;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ─── Input / Output types ───────────────────────────────────

#[derive(Deserialize, JsonSchema)]
struct WmPageGetInput {
    #[schemars(description = "Page ID")]
    id: String,
}

#[derive(Serialize)]
struct WmPageGetOutput {
    id: String,
    content: String,
    sections: Vec<PageSectionOutput>,
}

#[derive(Serialize)]
struct PageSectionOutput {
    header: String,
    body: String,
}

#[derive(Deserialize, JsonSchema)]
struct WmPageCreateInput {
    #[schemars(description = "Page ID (wiki path)")]
    path: String,
    #[schemars(description = "Page title")]
    title: String,
    #[schemars(description = "Page type: concept/task/spec/decision/pattern/howto/reference")]
    r#type: Option<String>,
    #[schemars(description = "Tags for the page")]
    tags: Option<Vec<String>>,
    #[schemars(description = "Page content (markdown)")]
    content: Option<String>,
    #[schemars(description = "Page status: draft/reviewed/approved")]
    status: Option<String>,
}

#[derive(Serialize)]
struct WmPageCreateOutput {
    id: String,
    path: String,
    r#type: String,
}

#[derive(Deserialize, JsonSchema)]
struct WmPageListInput {
    #[schemars(description = "Filter by page type")]
    r#type: Option<String>,
    #[schemars(description = "Max results")]
    limit: Option<i32>,
}

#[derive(Serialize)]
struct WmPageListOutput {
    pages: Vec<serde_json::Value>,
    total: usize,
}

#[derive(Deserialize, JsonSchema)]
struct WmPageUpdateInput {
    #[schemars(description = "Page ID")]
    id: String,
    #[schemars(description = "New title")]
    title: Option<String>,
    #[schemars(description = "New content")]
    content: Option<String>,
    #[schemars(description = "New status: draft/reviewed/approved")]
    status: Option<String>,
    #[schemars(description = "Tags")]
    tags: Option<Vec<String>>,
    #[schemars(description = "Page type")]
    r#type: Option<String>,
    #[schemars(description = "Related page edges")]
    relates_to: Option<Vec<serde_json::Value>>,
    #[schemars(description = "Implementation notes (replaces existing)")]
    notes: Option<String>,
    #[schemars(description = "Append to implementation notes")]
    append_notes: Option<String>,
}

#[derive(Serialize)]
struct WmPageUpdateOutput {
    id: String,
    status: String,
}

#[derive(Deserialize, JsonSchema)]
struct WmPageDeleteInput {
    #[schemars(description = "Page ID to delete")]
    id: String,
}

#[derive(Serialize)]
struct WmPageDeleteOutput {
    id: String,
    status: String,
}

#[derive(Deserialize, JsonSchema)]
struct WmPageLinkInput {
    #[schemars(description = "Source page ID")]
    source: String,
    #[schemars(description = "Target page ID")]
    target: String,
    #[schemars(description = "Edge type (e.g. relates_to, example_of)")]
    edge_type: Option<String>,
}

#[derive(Serialize)]
struct WmPageLinkOutput {
    id: String,
    target: String,
    r#type: String,
    status: String,
}

#[derive(Deserialize, JsonSchema)]
struct WmPageUnlinkInput {
    #[schemars(description = "Source page ID")]
    source: String,
    #[schemars(description = "Target page ID")]
    target: String,
    #[schemars(description = "Edge type to remove")]
    edge_type: String,
}

#[derive(Serialize)]
struct WmPageUnlinkOutput {
    id: String,
    target: String,
    status: String,
}

// ─── Tool Registration ──────────────────────────────────────

/// Register page tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_read(
        "wm_page.get",
        "Get page content by ID",
        move |input: WmPageGetInput| {
            let content = page::get_page(&e, &input.id)?;
            Ok(WmPageGetOutput {
                id: input.id,
                content: content.raw,
                sections: content
                    .sections
                    .iter()
                    .map(|s| PageSectionOutput {
                        header: s.header.clone(),
                        body: s.body.clone(),
                    })
                    .collect(),
            })
        },
    );

    let e = engine.clone();
    registry.register_write(
        "wm_page.create",
        "Create a new wiki page",
        move |input: WmPageCreateInput| {
            let content = input.content.unwrap_or_default();
            let page_type = input.r#type.unwrap_or_else(|| {
                let first_segment = input
                    .path
                    .trim_start_matches("wiki/")
                    .split('/')
                    .next()
                    .unwrap_or("concept");
                match first_segment {
                    "tasks" => "task".into(),
                    "specs" => "spec".into(),
                    "concepts" => "concept".into(),
                    "patterns" => "pattern".into(),
                    "decisions" => "decision".into(),
                    "howto" => "howto".into(),
                    "reference" => "reference".into(),
                    "notes" => "note".into(),
                    _ => "concept".into(),
                }
            });

            let mut frontmatter =
                format!("title: {}\ntype: {}\n", input.title, page_type);
            if let Some(status) = input.status {
                frontmatter.push_str(&format!("status: {}\n", status));
            }
            let id =
                page::create_page(&e, &input.path, &frontmatter, &content)?;
            let e2 = e.clone();
            e.index_scheduler.submit("page", move || {
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
                let memory_dir = root.join(".wm").join("memory");
                e2.rebuild_memory_index(&memory_dir);
            });
            Ok(WmPageCreateOutput {
                id,
                path: input.path,
                r#type: page_type,
            })
        },
    );

    let e = engine.clone();
    registry.register_read(
        "wm_page.list",
        "List all wiki pages",
        move |_input: WmPageListInput| {
            let pages = page::list_pages(&e)?;
            let total = pages.len();
            Ok(WmPageListOutput { pages, total })
        },
    );

    let e = engine.clone();
    registry.register_write(
        "wm_page.update",
        "Update page frontmatter fields",
        move |input: WmPageUpdateInput| {
            let mut params = serde_json::Map::new();
            params.insert("id".into(), json!(input.id));
            if let Some(title) = input.title {
                params.insert("title".into(), json!(title));
            }
            if let Some(content) = input.content {
                params.insert("content".into(), json!(content));
            }
            if let Some(status) = input.status {
                params.insert("status".into(), json!(status));
            }
            if let Some(tags) = input.tags {
                params.insert("tags".into(), json!(tags));
            }
            if let Some(r#type) = input.r#type {
                params.insert("type".into(), json!(r#type));
            }
            if let Some(relates_to) = input.relates_to {
                params.insert("relates_to".into(), json!(relates_to));
            }
            if let Some(notes) = input.notes {
                params.insert("implementation_notes".into(), json!(notes));
            }
            if let Some(append) = input.append_notes {
                params.insert("append_notes".into(), json!(append));
            }
            page::update_page(&e, &input.id, &serde_json::Value::Object(params))?;
            Ok(WmPageUpdateOutput {
                id: input.id,
                status: "updated".into(),
            })
        },
    );

    let e = engine.clone();
    registry.register_admin(
        "wm_page.delete",
        "Delete a page and its file",
        move |input: WmPageDeleteInput| {
            let snapshot = e.graph.load();
            let index = &snapshot.1;
            let node_idx = index
                .get(&input.id)
                .ok_or_else(|| ToolError::not_found("page", &input.id))?;
            let meta = &snapshot.0[*node_idx];
            let file_path = &meta.path;

            if file_path.exists() {
                std::fs::remove_file(file_path).map_err(|e| {
                    ToolError::internal(format!(
                        "Failed to delete {}: {}",
                        file_path.display(),
                        e
                    ))
                })?;
            }

            e.stale_flag
                .store(true, std::sync::atomic::Ordering::Release);
            Ok(WmPageDeleteOutput {
                id: input.id,
                status: "deleted".into(),
            })
        },
    );

    let e = engine.clone();
    registry.register_write(
        "wm_page.link",
        "Add a typed edge between pages",
        move |input: WmPageLinkInput| {
            let source = input.source;
            let target = input.target;
            let edge_type =
                input.edge_type.unwrap_or_else(|| "relates_to".into());
            let update = json!({
                "relates_to": [{"type": edge_type.as_str(), "target": target.as_str()}]
            });
            page::update_page(&e, &source, &update)?;
            Ok(WmPageLinkOutput {
                id: source,
                target,
                r#type: edge_type,
                status: "linked".into(),
            })
        },
    );

    let e = engine.clone();
    registry.register_write(
        "wm_page.unlink",
        "Remove a typed edge between pages",
        move |input: WmPageUnlinkInput| {
            let source = input.source;
            let target = input.target;
            let update = json!({
                "remove_relates_to": target.as_str()
            });
            page::update_page(&e, &source, &update)?;
            Ok(WmPageUnlinkOutput {
                id: source,
                target,
                status: "unlinked".into(),
            })
        },
    );
}
