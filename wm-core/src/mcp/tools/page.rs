use std::sync::Arc;

use serde_json::json;

use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;
use crate::page;

/// Register page tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_with_schema(
        "wm_page.get",
        "Get page content by ID",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Page ID" }
            },
            "required": ["id"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let content = page::get_page(&e, &id)?;
            Ok(serde_json::json!({
                "id": id,
                "content": content.raw,
                "sections": content.sections.iter().map(|s| {
                    serde_json::json!({ "header": s.header, "body": s.body })
                }).collect::<Vec<_>>(),
            }))
        }),
    );

    let e = engine.clone();
    registry.register_with_schema(
        "wm_page.create",
        "Create a new wiki page",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Page ID (wiki path)" },
                "title": { "type": "string", "description": "Page title" },
                "type": { "type": "string", "description": "Page type: concept/task/spec/decision/pattern/howto/reference" },
                "tags": { "type": "array", "items": { "type": "string" }, "description": "Tags for the page" },
                "content": { "type": "string", "description": "Page content (markdown)" },
                "status": { "type": "string", "description": "Page status: draft/reviewed/approved" }
            },
            "required": ["id", "title"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let path = args.require_string("path")?;
            let title = args.require_string("title")?;
            let content = args.optional_text("content").unwrap_or_default();
            let page_type = args.optional_string("type").unwrap_or_else(|| {
                let first_segment = path
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
                    _ => "concept".into(),
                }
            });

            let mut frontmatter = format!("title: {}\ntype: {}\n", title, page_type);
            if let Some(status) = args.optional_string("status") {
                frontmatter.push_str(&format!("status: {}\n", status));
            }
            let id = page::create_page(&e, &path, &frontmatter, &content)?;
            let e2 = e.clone();
            e.index_scheduler.submit("page", move || {
                let root = e2.project_root.read()
                    .map(|r| r.clone())
                    .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
                let wiki_dir = root.join(".wm").join("wiki");
                let sections = crate::graph::build_sections_from_wiki(&wiki_dir);
                let docs: Vec<crate::search::IndexedDoc> = sections.iter()
                    .map(|s| crate::search::IndexedDoc {
                        id: s.section_id.clone(),
                        fields: vec![
                            crate::search::Field::new("header", &s.header, 4.0),
                            crate::search::Field::new("body", &s.body, 1.0),
                        ],
                    }).collect();
                e2.bm25_index.store(Arc::new(crate::search::Bm25Index::build(docs)));
                let memory_dir = root.join(".wm").join("memory");
                e2.rebuild_memory_index(&memory_dir);
            });
            Ok(serde_json::json!({ "id": id, "path": path, "type": page_type }))
        }),
    );

    let e = engine.clone();
    registry.register_with_schema(
        "wm_page.list",
        "List all wiki pages",
        json!({
            "type": "object",
            "properties": {
                "type": { "type": "string", "description": "Filter by page type" },
                "limit": { "type": "integer", "description": "Max results" }
            }
        }),
        Arc::new(move |_params| {
            let pages = page::list_pages(&e)?;
            Ok(serde_json::json!({ "pages": pages, "total": pages.len() }))
        }),
    );

    let e = engine.clone();
    registry.register_with_schema(
        "wm_page.update",
        "Update page frontmatter fields",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Page ID" },
                "title": { "type": "string", "description": "New title" },
                "content": { "type": "string", "description": "New content" },
                "status": { "type": "string", "description": "New status: draft/reviewed/approved" },
                "tags": { "type": "array", "items": { "type": "string" }, "description": "Tags" },
                "type": { "type": "string", "description": "Page type" },
                "relates_to": { "type": "array", "items": { "type": "object" }, "description": "Related page edges" }
            },
            "required": ["id"]
        }),
        Arc::new(move |params: serde_json::Value| {
            let args = ToolArgs::new(params.clone());
            let id = args.require_string("id")?;
            page::update_page(&e, &id, &params)?;
            Ok(serde_json::json!({ "id": id, "status": "updated" }))
        }),
    );

    let e = engine.clone();
    registry.register_with_schema(
        "wm_page.delete",
        "Delete a page and its file",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Page ID to delete" }
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
                .ok_or_else(|| ToolError::not_found("page", &id))?;
            let meta = &snapshot.0[*node_idx];
            let file_path = &meta.path;

            if file_path.exists() {
                std::fs::remove_file(file_path).map_err(|e| {
                    ToolError::internal(format!("Failed to delete {}: {}", file_path.display(), e))
                })?;
            }

            e.stale_flag
                .store(true, std::sync::atomic::Ordering::Release);
            Ok(serde_json::json!({ "id": id, "status": "deleted" }))
        }),
    );

    let e = engine.clone();
    registry.register_with_schema("wm_page.link", "Add a typed edge between pages", json!({
        "type": "object",
        "properties": {
            "source": { "type": "string", "description": "Source page ID" },
            "target": { "type": "string", "description": "Target page ID" },
            "edge_type": { "type": "string", "description": "Edge type (e.g. relates_to, example_of)" }
        },
        "required": ["source", "target", "edge_type"]
    }), Arc::new(move |params| {
        let args = ToolArgs::new(params);
        let source = args.require_string("source")?;
        let target = args.require_string("target")?;
        let edge_type = args.optional_string("edge_type").unwrap_or_else(|| "relates_to".into());

        let update = serde_json::json!({
            "relates_to": [{"type": edge_type, "target": target}]
        });
        page::update_page(&e, &source, &update)?;
        Ok(serde_json::json!({ "id": source, "target": target, "type": edge_type, "status": "linked" }))
    }));

    let e = engine.clone();
    registry.register_with_schema(
        "wm_page.unlink",
        "Remove a typed edge between pages",
        json!({
            "type": "object",
            "properties": {
                "source": { "type": "string", "description": "Source page ID" },
                "target": { "type": "string", "description": "Target page ID" },
                "edge_type": { "type": "string", "description": "Edge type to remove" }
            },
            "required": ["source", "target", "edge_type"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let source = args.require_string("source")?;
            let target = args.require_string("target")?;

            let update = serde_json::json!({
                "remove_relates_to": target
            });
            page::update_page(&e, &source, &update)?;
            Ok(serde_json::json!({ "id": source, "target": target, "status": "unlinked" }))
        }),
    );
}
