use std::sync::Arc;

use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;
use crate::page;

/// Register page tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_with_desc(
        "wm_page.get",
        "Get page content by ID",
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
    registry.register_with_desc(
        "wm_page.create",
        "Create a new wiki page",
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

            let frontmatter = format!("title: {}\ntype: {}\n", title, page_type);
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
    registry.register_with_desc(
        "wm_page.list",
        "List all wiki pages",
        Arc::new(move |_params| {
            let pages = page::list_pages(&e)?;
            Ok(serde_json::json!({ "pages": pages, "total": pages.len() }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_page.update",
        "Update page frontmatter fields",
        Arc::new(move |params: serde_json::Value| {
            let args = ToolArgs::new(params.clone());
            let id = args.require_string("id")?;
            page::update_page(&e, &id, &params)?;
            Ok(serde_json::json!({ "id": id, "status": "updated" }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_page.delete",
        "Delete a page and its file",
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
    registry.register_with_desc("wm_page.link", "Add a typed edge between pages", Arc::new(move |params| {
        let args = ToolArgs::new(params);
        let id = args.require_string("id")?;
        let target = args.require_string("target")?;
        let edge_type = args.optional_string("type").unwrap_or_else(|| "relates_to".into());

        let update = serde_json::json!({
            "relates_to": [{"type": edge_type, "target": target}]
        });
        page::update_page(&e, &id, &update)?;
        Ok(serde_json::json!({ "id": id, "target": target, "type": edge_type, "status": "linked" }))
    }));

    let e = engine.clone();
    registry.register_with_desc(
        "wm_page.unlink",
        "Remove a typed edge between pages",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let target = args.require_string("target")?;

            let update = serde_json::json!({
                "remove_relates_to": target
            });
            page::update_page(&e, &id, &update)?;
            Ok(serde_json::json!({ "id": id, "target": target, "status": "unlinked" }))
        }),
    );
}
