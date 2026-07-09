use std::sync::Arc;

use serde_json::json;

use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;

/// Register search tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_with_schema("wm_search.query", "Search the wiki and/or memory (keyword/semantic/hybrid)", json!({
        "type": "object",
        "properties": {
            "q": { "type": "string", "description": "Search query" },
            "type": { "type": "string", "description": "Search type: all/page/memory", "default": "all" },
            "mode": { "type": "string", "description": "Search mode: auto/keyword/semantic/hybrid", "default": "auto" },
            "limit": { "type": "integer", "description": "Max results", "default": 10 },
            "offset": { "type": "integer", "description": "Result offset", "default": 0 }
        },
        "required": ["q"]
    }), Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let query = args.require_string("q")?;
            let limit = args.optional_int("limit").unwrap_or(10) as usize;
            let offset = args.optional_int("offset").unwrap_or(0) as usize;
            let r#type = args.optional_string("type").unwrap_or_else(|| "all".into());
            let mode_str = args.optional_string("mode").unwrap_or_else(|| "auto".into());

            let start = std::time::Instant::now();
            let embedder_loaded = e.embedder.is_loaded();

            let params = crate::search::QueryParams {
                query: query.clone(),
                r#type,
                mode: mode_str,
                limit,
                offset,
                recency: true,
            };

            let results = crate::search::query(&e, &params)
                .map_err(|msg| ToolError::internal(msg))?;

            let json_results: Vec<serde_json::Value> = results.into_iter().map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "score": r.score,
                    "type": r.r#type,
                    "page_type": r.page_type,
                    "page_type_rank": r.page_type_rank,
                    "centrality": r.centrality,
                    "snippet": r.snippet,
                })
            }).collect();

            let elapsed = start.elapsed().as_millis() as i64;
            Ok(serde_json::json!({
                "query": query,
                "mode": if params.mode == "auto" {
                    crate::embed::SearchMode::auto_detect(&query).to_string()
                } else {
                    crate::embed::SearchMode::from_str(&params.mode).to_string()
                },
                "embedder_loaded": embedder_loaded,
                "search_time_ms": elapsed,
                "results": json_results,
                "total": json_results.len(),
            }))
        })
    );

    let e = engine.clone();
    registry.register_with_schema(
        "wm_search.retrieve",
        "Context assembly with token budget (type: all/page/memory)",
        json!({
            "type": "object",
            "properties": {
                "q": { "type": "string", "description": "Search query" },
                "token_budget": { "type": "integer", "description": "Token budget for context", "default": 8192 },
                "type": { "type": "string", "description": "Source type: all/page/memory", "default": "all" }
            },
            "required": ["q"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let query = args.require_string("q")?;
            let token_budget = args.optional_int("token_budget").unwrap_or(8192);
            let r#type = args.optional_string("type").unwrap_or_else(|| "all".into());

            let search_pages = r#type == "all" || r#type == "page" || r#type == "task";
            let search_memory = r#type == "all" || r#type == "memory";

            let mut context_text = String::new();
            let mut results: Vec<serde_json::Value> = Vec::new();

            if search_pages {
                let snapshot = e.graph.load();
                let graph = &snapshot.0;
                let index = &snapshot.1;

                let qp = crate::search::QueryParams {
                    query: query.clone(),
                    r#type: "page".into(),
                    mode: "auto".into(),
                    limit: 1,
                    offset: 0,
                    recency: false,
                };
                let qr = crate::search::query(&e, &qp).unwrap_or_default();
                let bfs_seed = qr.first().map(|r| r.id.clone()).unwrap_or_else(|| query.clone());

                let page_ctx = crate::search::retrieve_context(graph, index, &bfs_seed, token_budget, None);
                context_text.push_str(&page_ctx.iter().map(|(_, _, text)| text.as_str()).collect::<Vec<&str>>().join("\n"));
                for (id, score, _) in &page_ctx {
                    let page_type = index
                        .get(id)
                        .map(|&idx| &graph[idx])
                        .map(|meta| format!("{:?}", meta.page_type).to_lowercase())
                        .unwrap_or_default();
                    results.push(serde_json::json!({
                        "id": id, "score": score, "page_type": page_type, "type": "page",
                    }));
                }
            }

            if search_memory {
                let qp = crate::search::QueryParams {
                    query: query.clone(),
                    r#type: "memory".into(),
                    mode: "auto".into(),
                    limit: 10,
                    offset: 0,
                    recency: false,
                };
                if let Ok(mem_results) = crate::search::query(&e, &qp) {
                    for r in &mem_results {
                        if let Some(sep) = r.id.find(':') {
                            let mem_id = &r.id[sep+1..];
                            if mem_id.contains("..") || mem_id.contains('/') || mem_id.contains('\\') {
                                continue;
                            }
                            let root = e.project_root.read().map_err(|_| ToolError::lock_poisoned("project_root"))?;
                            let mem_path = root.join(".wm").join("memory").join(format!("{}.json", mem_id));
                            drop(root);
                            if let Ok(content) = std::fs::read_to_string(&mem_path) {
                                if let Ok(mem) = serde_json::from_str::<crate::engine::MemoryEntry>(&content) {
                                    let text = format!("[memory:{}] {} — {}\n", mem.id, mem.title, mem.content);
                                    if context_text.len() + text.len() <= token_budget {
                                        context_text.push_str(&text);
                                        results.push(serde_json::json!({
                                            "id": r.id, "score": r.score, "type": "memory",
                                        }));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Ok(serde_json::json!({
                "query": query,
                "token_budget": token_budget,
                "tokens_used": context_text.len() / 4,
                "result_count": results.len(),
                "results": results,
                "context": context_text,
            }))
        }),
    );

    let e = engine.clone();
    registry.register_with_schema(
        "wm_search.resolve",
        "Resolve a query to a page ID",
        json!({
            "type": "object",
            "properties": {
                "q": { "type": "string", "description": "Query to resolve" }
            },
            "required": ["q"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let q = args.require_string("q")?;

            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let index = &snapshot.1;

            if let Some(&idx) = index.get(&q) {
                let meta = &graph[idx];
                return Ok(serde_json::json!({
                    "resolved": true,
                    "id": meta.id,
                    "title": meta.title,
                    "page_type": format!("{:?}", meta.page_type).to_lowercase(),
                }));
            }

            for idx in graph.node_indices() {
                let meta = &graph[idx];
                if meta.title.eq_ignore_ascii_case(&q) || meta.id.eq_ignore_ascii_case(&q) {
                    return Ok(serde_json::json!({
                        "resolved": true,
                        "id": meta.id,
                        "title": meta.title,
                        "page_type": format!("{:?}", meta.page_type).to_lowercase(),
                    }));
                }
            }

            let bm25 = e.bm25_index.load();
            let bm25_results = bm25.search(&q, 5);
            if !bm25_results.is_empty() {
                let candidates: Vec<serde_json::Value> = bm25_results.iter().map(|r| {
                serde_json::json!({ "id": r.id, "score": r.score, "snippet": r.snippet })
            }).collect();
                return Ok(serde_json::json!({
                    "resolved": false,
                    "candidates": candidates,
                    "total": candidates.len(),
                }));
            }

            Ok(serde_json::json!({
                "resolved": false,
                "candidates": [],
                "total": 0,
            }))
        }),
    );
}
