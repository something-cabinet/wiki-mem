use petgraph::visit::EdgeRef;
use std::sync::Arc;

use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::handler::ToolArgs;
use crate::page;
use crate::search::{IndexedDoc, Field};
use crate::source;

/// Register all MCP tool handlers on the engine
pub fn register_all_tools(
    registry: &mut crate::mcp::transport::ToolRegistry,
    engine: Arc<EngineState>,
) {
    // ─── Initial Tool ────────────────────────────────────────

    let e = engine.clone();
    registry.register("wm_initial", Arc::new(move |_params| {
        let snapshot = e.graph.load();
        let node_count = snapshot.0.node_count();
        Ok(serde_json::json!({
            "project": "active",
            "graph_nodes": node_count,
            "instructions": "Wiki Memory Engine — use wm_* tools for all operations.",
            "model_status": if node_count > 0 { "graph_loaded" } else { "empty_project" }
        }))
    }));

    // ─── Help Tool ───────────────────────────────────────────

    registry.register("wm_help", Arc::new(|params| {
        let _args = ToolArgs::new(params);
        Ok(serde_json::json!({
            "available_tools": [
                "wm_initial", "wm_help",
                "wm_page.{create,get,update,delete,list}",
                "wm_search.{query,retrieve}",
                "wm_source.{add,process,complete,list,verify}",
                "wm_graph.neighbors",
                "wm_lint.check", "wm_validate.check",
                "wm_index.rebuild"
            ],
            "documentation": "Use wm_help to get started with any tool."
        }))
    }));

    // ─── Search Tools ─────────────────────────────────────────

    let e = engine.clone();
    registry.register("wm_search.query", Arc::new(move |params| {
        let args = ToolArgs::new(params);
        let query = args.require_string("q")?;
        let limit = args.optional_int("limit").unwrap_or(10);

        let bm25 = e.bm25_index.load();
        let results = bm25.search(&query, limit);

        let results: Vec<serde_json::Value> = results.iter().map(|r| {
            serde_json::json!({
                "id": r.id,
                "score": r.score,
                "snippet": r.snippet,
            })
        }).collect();

        Ok(serde_json::json!({
            "query": query,
            "mode": "keyword",
            "results": results,
            "total": results.len(),
        }))
    }));

    let e = engine.clone();
    registry.register("wm_search.retrieve", Arc::new(move |params| {
        let args = ToolArgs::new(params);
        let query = args.require_string("q")?;
        let token_budget = args.optional_int("token_budget").unwrap_or(8192);

        let snapshot = e.graph.load();
        let graph = &snapshot.0;
        let index = &snapshot.1;

        let context = crate::search::retrieve_context(graph, index, &query, token_budget);

        let context_text: String = context.iter()
            .map(|(_, _, text)| text.as_str())
            .collect::<Vec<&str>>()
            .join("\n");

        let result_ids: Vec<String> = context.iter().map(|(id, _, _)| id.clone()).collect();

        Ok(serde_json::json!({
            "query": query,
            "token_budget": token_budget,
            "tokens_used": context_text.len() / 4,
            "result_count": context.len(),
            "results": result_ids,
            "context": context_text,
        }))
    }));

    // ─── Page Tools ───────────────────────────────────────────

    let e = engine.clone();
    registry.register("wm_page.get", Arc::new(move |params| {
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
    }));

    let e = engine.clone();
    registry.register("wm_page.create", Arc::new(move |params| {
        let args = ToolArgs::new(params);
        let path = args.require_string("path")?;
        let title = args.require_string("title")?;
        let content = args.optional_text("content").unwrap_or_default();

        let frontmatter = format!("title: {}\ntype: concept\n", title);
        let id = page::create_page(&e, &path, &frontmatter, &content)?;
        e.stale_flag.store(true, std::sync::atomic::Ordering::Release);
        Ok(serde_json::json!({ "id": id, "path": path }))
    }));

    let e = engine.clone();
    registry.register("wm_page.list", Arc::new(move |_params| {
        let pages = page::list_pages(&e)?;
        Ok(serde_json::json!({ "pages": pages, "total": pages.len() }))
    }));

    let e = engine.clone();
    registry.register("wm_page.update", Arc::new(move |params: serde_json::Value| {
        let args = ToolArgs::new(params.clone());
        let id = args.require_string("id")?;
        page::update_page(&e, &id, &params)?;
        Ok(serde_json::json!({ "id": id, "status": "updated" }))
    }));

    // ─── Source Tools ─────────────────────────────────────────

    let e = engine.clone();
    registry.register("wm_source.add", Arc::new(move |params| {
        let args = ToolArgs::new(params);
        let path = args.require_string("path")?;
        let id = source::add_source(&e, &path)?;
        Ok(serde_json::json!({ "id": id, "state": "pending" }))
    }));

    let e = engine.clone();
    registry.register("wm_source.process", Arc::new(move |params| {
        let args = ToolArgs::new(params);
        let id = args.require_string("id")?;
        let content = source::process_source(&e, &id)?;
        Ok(serde_json::json!({ "id": id, "content": content }))
    }));

    let e = engine.clone();
    registry.register("wm_source.complete", Arc::new(move |params| {
        let args = ToolArgs::new(params);
        let id = args.require_string("id")?;
        let refs = args.optional_string_array("page_refs");
        source::complete_source(&e, &id, &refs)?;
        Ok(serde_json::json!({ "id": id, "status": "done", "pages": refs.len() }))
    }));

    let e = engine.clone();
    registry.register("wm_source.error", Arc::new(move |params| {
        let args = ToolArgs::new(params);
        let id = args.require_string("id")?;
        let msg = args.optional_string("message").unwrap_or_else(|| "Unknown error".to_string());
        source::error_source(&e, &id, &msg)?;
        Ok(serde_json::json!({ "id": id, "status": "error", "message": msg }))
    }));

    let e = engine.clone();
    registry.register("wm_source.list", Arc::new(move |params| {
        let args = ToolArgs::new(params);
        let state = args.optional_string("state");
        let sources = source::list_sources(&e, state.as_deref())?;
        Ok(serde_json::json!({ "sources": sources, "total": sources.len() }))
    }));

    let e = engine.clone();
    registry.register("wm_source.verify", Arc::new(move |params| {
        let args = ToolArgs::new(params);
        let id = args.require_string("id")?;
        let is_stale = source::verify_source(&e, &id)?;
        Ok(serde_json::json!({ "id": id, "stale": is_stale }))
    }));

    let e = engine.clone();
    registry.register("wm_source.discover", Arc::new(move |_params| {
        let dirs = {
            let config = e.config.read().unwrap();
            config.source_dirs.clone()
        };
        let discovered = source::discover_sources(&e, &dirs)?;
        Ok(serde_json::json!({ "discovered": discovered, "total": discovered.len() }))
    }));

    // ─── Graph Tools ──────────────────────────────────────────

    let e = engine.clone();
    registry.register("wm_graph.neighbors", Arc::new(move |params| {
        let args = ToolArgs::new(params);
        let id = args.require_string("id")?;
        let query = args.optional_string("query");

        let snapshot = e.graph.load();
        let graph = &snapshot.0;
        let index = &snapshot.1;

        let start = index.get(&id).ok_or_else(|| ToolError::not_found("page", &id))?;
        let mut neighbors = Vec::new();

        for edge in graph.edges(*start) {
            let target = edge.target();
            let edge_type = edge.weight();
            let meta = &graph[target];

            let score = if let Some(ref q) = query {
                let bm25 = crate::search::Bm25Index::build(vec![
                    IndexedDoc {
                        id: meta.id.clone(),
                        fields: vec![
                            Field::new("title", &meta.title, 4.0),
                            Field::new("tags", &meta.tags.join(" "), 2.2),
                        ],
                    }
                ]);
                let results = bm25.search(q, 1);
                let bm25_score = results.first().map(|r| r.score).unwrap_or(0.0);
                edge_type.priority() as f64 * bm25_score
            } else {
                edge_type.priority() as f64
            };

            neighbors.push(serde_json::json!({
                "id": meta.id,
                "title": meta.title,
                "edge_type": format!("{:?}", edge_type).to_lowercase(),
                "score": score,
            }));
        }

        neighbors.sort_by(|a, b| {
            let sa = a["score"].as_f64().unwrap_or(0.0);
            let sb = b["score"].as_f64().unwrap_or(0.0);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(serde_json::json!({
            "id": id,
            "neighbors": neighbors,
            "total": neighbors.len(),
        }))
    }));

    // ─── Lint Tools ───────────────────────────────────────────

    let e = engine.clone();
    registry.register("wm_lint.check", Arc::new(move |_params| {
        let snapshot = e.graph.load();
        let graph = &snapshot.0;
        let mut issues = Vec::new();

        for idx in graph.node_indices() {
            let has_inbound = graph.edges_directed(idx, petgraph::Direction::Incoming).count() > 0;
            if !has_inbound {
                let meta = &graph[idx];
                issues.push(serde_json::json!({
                    "type": "orphan",
                    "id": meta.id,
                    "message": "No inbound edges — consider adding relationships"
                }));
            }
        }

        Ok(serde_json::json!({ "issues": issues, "total": issues.len() }))
    }));

    // ─── Validate Tools ───────────────────────────────────────

    let e = engine.clone();
    registry.register("wm_validate.check", Arc::new(move |_params| {
        let snapshot = e.graph.load();
        let graph = &snapshot.0;
        let mut errors: Vec<String> = Vec::new();

        for idx in graph.node_indices() {
            let meta = &graph[idx];
            if meta.title.is_empty() {
                errors.push(format!("Page {} has no title", meta.id));
            }
        }

        Ok(serde_json::json!({
            "status": if errors.is_empty() { "pass" } else { "fail" },
            "nodes": graph.node_count(),
            "edges": graph.edge_count(),
            "errors": errors,
        }))
    }));

    // ─── Index Rebuild ────────────────────────────────────────

    let e = engine.clone();
    registry.register("wm_index.rebuild", Arc::new(move |_params| {
        let root = std::env::current_dir()
            .map_err(|e| ToolError::internal(e.to_string()))?;
        let wiki_dir = root.join(".wm").join("wiki");

        if !wiki_dir.exists() {
            return Err(ToolError::internal("No wiki directory found. Run 'wm init' first."));
        }

        // 1. Rebuild graph
        let count = crate::graph::rebuild_snapshot(&e.graph, &wiki_dir);

        // 2. Rebuild section corpus
        let sections = crate::graph::build_sections_from_wiki(&wiki_dir);
        e.section_corpus.store(Arc::new(sections.clone()));

        // 3. Rebuild BM25 index
        let docs: Vec<crate::search::IndexedDoc> = sections.iter().map(|s| {
            crate::search::IndexedDoc {
                id: s.section_id.clone(),
                fields: vec![
                    crate::search::Field::new("header", &s.header, 4.0),
                    crate::search::Field::new("body", &s.body, 1.0),
                ],
            }
        }).collect();
        let bm25 = crate::search::Bm25Index::build(docs);
        e.bm25_index.store(Arc::new(bm25));

        e.stale_flag.store(false, std::sync::atomic::Ordering::Release);

        Ok(serde_json::json!({
            "status": "ok",
            "graph_nodes": count,
            "sections": sections.len(),
            "message": "Full rebuild complete"
        }))
    }));

    // ─── Task Tools ───────────────────────────────────────────

    let e = engine.clone();
    registry.register("wm_task.check_ac", Arc::new(move |params| {
        let args = ToolArgs::new(params);
        let id = args.require_string("id")?;
        let ac_indices = args.optional_string_array("criteria");
        let indices: Vec<u64> = ac_indices.iter().filter_map(|s| s.parse().ok()).collect();
        let update = serde_json::json!({ "checked_ac": indices });
        page::update_page(&e, &id, &update)?;
        Ok(serde_json::json!({ "id": id, "checked": indices }))
    }));

    let e = engine.clone();
    registry.register("wm_task.uncheck_ac", Arc::new(move |params| {
        let args = ToolArgs::new(params);
        let id = args.require_string("id")?;
        let ac_indices = args.optional_string_array("criteria");
        let indices: Vec<u64> = ac_indices.iter().filter_map(|s| s.parse().ok()).collect();
        let update = serde_json::json!({ "unchecked_ac": indices });
        page::update_page(&e, &id, &update)?;
        Ok(serde_json::json!({ "id": id, "unchecked": indices }))
    }));

    // ─── Log Tools ────────────────────────────────────────────

    registry.register("wm_log.recent", Arc::new(|params| {
        let args = ToolArgs::new(params);
        let count = args.optional_int("count").unwrap_or(20);
        let log_path = std::path::Path::new(".wm").join("wiki").join("log.md");
        let content = std::fs::read_to_string(&log_path).unwrap_or_default();
        let all_lines: Vec<&str> = content.lines().collect();
        let total = all_lines.len();
        let start = total.saturating_sub(count);
        let lines: Vec<&str> = all_lines[start..].to_vec();
        Ok(serde_json::json!({
            "entries": lines,
            "total": total,
        }))
    }));

    // ─── Project Tools ────────────────────────────────────────

    registry.register("wm_project.status", Arc::new(|_params| {
        let root = std::env::current_dir().ok();
        let project = root.as_ref().and_then(|r| {
            let config_path = r.join(".wm").join("config.json");
            std::fs::read_to_string(config_path).ok()
        });
        Ok(serde_json::json!({
            "project": if project.is_some() { "active" } else { "none" },
            "root": root.map(|r| r.to_string_lossy().to_string()),
        }))
    }));
}

