use petgraph::visit::EdgeRef;
use std::sync::Arc;

use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::handler::ToolArgs;
use crate::page;
use crate::search::Bm25Index;
use crate::source;

/// Parse a duration string like "2h 30m" or "45m" into total minutes.
fn parse_duration_to_minutes(s: &str) -> f64 {
    let s = s.trim();
    if s.is_empty() { return 0.0; }
    let mut minutes = 0.0;
    if let Some(h) = s.split('h').next().and_then(|p| p.trim().parse::<f64>().ok()) {
        minutes += h * 60.0;
    }
    if let Some(m_part) = s.rsplit('h').next() {
        if let Ok(m) = m_part.trim().trim_end_matches('m').parse::<f64>() {
            minutes += m;
        }
    }
    minutes
}

use sha2::{Digest, Sha256};

/// Register all MCP tool handlers on the engine
pub fn register_all_tools(
    registry: &mut crate::mcp::transport::ToolRegistry,
    engine: Arc<EngineState>,
) {
    // ─── Initial Tool ────────────────────────────────────────

    let e = engine.clone();
    registry.register_with_desc(
        "wm_initial",
        "Get project state, graph stats, and model status",
        Arc::new(move |_params| {
            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let node_count = graph.node_count();
            let model_loaded = e.embedder.is_loaded();

            // Per-type page counts
            let mut page_types: std::collections::BTreeMap<String, usize> =
                std::collections::BTreeMap::new();
            for idx in graph.node_indices() {
                let type_name = format!("{:?}", graph[idx].page_type).to_lowercase();
                *page_types.entry(type_name).or_insert(0) += 1;
            }

            // Per-state source counts
            let mut source_states: std::collections::BTreeMap<String, usize> =
                std::collections::BTreeMap::new();
            let registry = e.source_registry.read().map_err(|_| ToolError::internal("registry lock poisoned"))?;
            for entry in registry.values() {
                let state_name = format!("{:?}", entry.state).to_lowercase();
                *source_states.entry(state_name).or_insert(0) += 1;
            }

            let sections = e.section_corpus.load();
            let bm25_loaded = e.bm25_index.load().total_docs > 0;

            Ok(serde_json::json!({
                "project": "active",
                "graph_nodes": node_count,
                "graph_edges": graph.edge_count(),
                "pages_by_type": page_types,
                "sources_by_state": source_states,
                "sections_indexed": sections.len(),
                "bm25_loaded": bm25_loaded,
                "instructions": "Wiki Memory Engine — use wm_* tools for all operations.",
                "embedding": {
                    "loaded": model_loaded,
                    "model_name": e.embedder.model_name(),
                    "dimensions": e.embedder.output_dim(),
                    "vectors_loaded": e.vector_store.snapshot().len(),
                },
                "search_modes_available": if model_loaded {
                    vec!["keyword", "semantic", "hybrid"]
                } else {
                    vec!["keyword"]
                }
            }))
        }),
    );

    // ─── Help Tool ───────────────────────────────────────────

    registry.register_with_desc("wm_help", "Search tool documentation (optional: q=pattern)", Arc::new(|params| {
        let args = ToolArgs::new(params);
        let q = args.optional_string("q");

        let all_tools = [
            ("wm_initial", "Get project state, graph stats, and model status"),
            ("wm_help", "Search tool documentation (optional: q=pattern)"),
            ("wm_search.query", "Search the wiki (keyword/semantic/hybrid)"),
            ("wm_search.retrieve", "Context assembly with token budget"),
            ("wm_search.resolve", "Resolve a query to a page ID"),
            ("wm_page.get", "Get page content by ID"),
            ("wm_page.create", "Create a new wiki page"),
            ("wm_page.update", "Update page frontmatter fields"),
            ("wm_page.delete", "Delete a page and its file"),
            ("wm_page.list", "List all wiki pages"),
            ("wm_page.link", "Add a typed edge between pages"),
            ("wm_page.unlink", "Remove a typed edge between pages"),
            ("wm_source.add", "Add a raw source file to the registry"),
            ("wm_source.process", "Process a source (pending→processing)"),
            ("wm_source.complete", "Complete source processing (processing→done)"),
            ("wm_source.error", "Mark a source as errored"),
            ("wm_source.list", "List sources with optional state filter"),
            ("wm_source.verify", "Verify source staleness by hash"),
            ("wm_source.discover", "Scan configured directories for new sources"),
            ("wm_source.remove", "Remove a source from the registry"),
            ("wm_source.status", "Get detailed source status"),
            ("wm_graph.neighbors", "Get typed edges from a page"),
            ("wm_graph.stats", "Graph statistics (node/edge counts by type)"),
            ("wm_graph.path", "Find shortest path between two pages"),
            ("wm_graph.subgraph", "Get neighborhood around a page node"),
            ("wm_task.check_ac", "Check an acceptance criterion"),
            ("wm_task.uncheck_ac", "Uncheck an acceptance criterion"),
            ("wm_task.board", "Task board grouped by status"),
            ("wm_time.start", "Start time tracking on a task"),
            ("wm_time.stop", "Stop time tracking, record elapsed"),
            ("wm_time.add", "Manually add time to a task"),
            ("wm_time.report", "Time report across all tasks"),
            ("wm_index.rebuild", "Full rebuild (graph + BM25 + embeddings)"),
            ("wm_index.embed", "Build embedding vectors only"),
            ("wm_index.status", "Show index state (sections, vectors, stale)"),
            ("wm_model.list", "List cached and available models"),
            ("wm_model.status", "Show current model state"),
            ("wm_model.download", "Download an embedding model"),
            ("wm_model.remove", "Remove a cached model"),
            ("wm_lint.check", "Check wiki for common issues"),
            ("wm_lint.fix", "Auto-fix common issues"),
            ("wm_validate.check", "Validate wiki health"),
            ("wm_log.recent", "Recent log entries"),
            ("wm_log.since", "Log entries since a marker"),
            ("wm_log.filter", "Filter log entries by text"),
            ("wm_project.status", "Project status information"),
            ("wm_skill.*", "Registered skill workflows"),
        ];

        let matched: Vec<serde_json::Value> = match q {
            Some(ref query) => {
                let q_lower = query.to_lowercase();
                // Wildcard prefix: "page.*" should match "wm_page.*"
                let prefix = q_lower.strip_suffix(".*").or_else(|| q_lower.strip_suffix('*'));
                all_tools.iter()
                    .filter(|(name, desc)| {
                        // Wildcard prefix match
                        if let Some(p) = prefix {
                            return name.to_lowercase().contains(p);
                        }
                        // Exact or substring match
                        name.to_lowercase().contains(&q_lower) || desc.to_lowercase().contains(&q_lower)
                    })
                    .map(|(name, desc)| serde_json::json!({ "name": name, "description": desc }))
                    .collect()
            }
            None => {
                all_tools.iter()
                    .map(|(name, desc)| serde_json::json!({ "name": name, "description": desc }))
                    .collect()
            }
        };

        Ok(serde_json::json!({
            "available_tools": matched,
            "total": matched.len(),
            "documentation": "Use wm_help with q=<search> to filter tools by name or description."
        }))
    }));

    // ─── Search Tools ─────────────────────────────────────────

    let e = engine.clone();
    registry.register_with_desc("wm_search.query", "Search the wiki and/or memory (keyword/semantic/hybrid)", Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let query = args.require_string("q")?;
            let limit = args.optional_int("limit").unwrap_or(10);
            let r#type = args.optional_string("type").unwrap_or_else(|| "all".into());
            let mode_str = args.optional_string("mode").unwrap_or_else(|| "auto".into());
            let mode = if mode_str == "auto" {
                crate::embed::SearchMode::auto_detect(&query)
            } else {
                crate::embed::SearchMode::from_str(&mode_str)
            };

            let start = std::time::Instant::now();
            let embedder_loaded = e.embedder.is_loaded();

            // Snapshot for page_type enrichment
            let snap = e.graph.load();
            let graph = &snap.0;
            let id_index = &snap.1;

            let search_pages = r#type == "all" || r#type == "page" || r#type == "task";
            let search_memory = r#type == "all" || r#type == "memory";

            // Acquire config once, extract owned values
            let config_guard = e.config.read().map_err(|_| ToolError::internal("config lock poisoned"))?;
            let rrf_k = config_guard.search.rrf_k as f64;
            let recency_model = config_guard.search.scoring.recency_model.clone();
            let recency_stability = config_guard.search.scoring.recency_stability_days as f64;
            let memory_salience_boost = config_guard.search.scoring.memory_salience_boost;
            let memory_salience_clamp = config_guard.search.scoring.memory_salience_clamp;
            drop(config_guard);

            let mut all_results: Vec<serde_json::Value> = Vec::new();
            let mut mode_used = "keyword".to_string();

            // 1. Search pages
            if search_pages {
                let (page_results, pmode): (Vec<serde_json::Value>, &str) = match mode {
                    crate::embed::SearchMode::Keyword => {
                        let bm25 = e.bm25_index.load();
                        let r = bm25.search(&query, limit);
                        (r.iter().map(|r| serde_json::json!({
                            "id": r.id, "score": r.score, "snippet": r.snippet,
                        })).collect(), "keyword")
                    }
                    crate::embed::SearchMode::Semantic => {
                        if !embedder_loaded {
                            return Err(ToolError::internal("Semantic search unavailable: no embedding model loaded. Run 'wm model download' first."));
                        }
                        let vectors = e.vector_store.snapshot();
                        if vectors.is_empty() {
                            return Err(ToolError::internal("No embeddings indexed. Run 'wm index embed' first."));
                        }
                        let query_vec = e.embedder.embed(&query)
                            .map_err(|e| ToolError::internal(format!("Embedding failed: {}", e)))?;
                        let top_k = crate::embed::top_k_cosine(&query_vec.0, &vectors, limit);
                        (top_k.into_iter().map(|(id, score)| serde_json::json!({
                            "id": id, "score": score, "snippet": "",
                        })).collect(), "semantic")
                    }
                    crate::embed::SearchMode::Hybrid => {
                        if !embedder_loaded {
                            let bm25 = e.bm25_index.load();
                            let r = bm25.search(&query, limit);
                            (r.iter().map(|r| serde_json::json!({
                                "id": r.id, "score": r.score, "snippet": r.snippet,
                            })).collect(), "keyword")
                        } else {
                            let vectors = e.vector_store.snapshot();
                            let bm25 = e.bm25_index.load();
                            let bm25_results = bm25.search(&query, limit * 2);
                            let bm25_pairs: Vec<(String, f64)> = bm25_results.iter()
                                .map(|r| (r.id.clone(), r.score)).collect();

                            let query_vec = e.embedder.embed(&query)
                                .map_err(|e| ToolError::internal(format!("Embedding failed: {}", e)))?;
                            let semantic_pairs = if vectors.is_empty() {
                                Vec::new()
                            } else {
                                crate::embed::top_k_cosine(&query_vec.0, &vectors, limit * 2)
                            };

                            let fused = crate::embed::rrf_fusion(&bm25_pairs, &semantic_pairs, rrf_k);
                            let truncated: Vec<_> = fused.into_iter().take(limit).collect();
                            (truncated.into_iter().map(|(id, score)| serde_json::json!({
                                "id": id, "score": score, "snippet": "",
                            })).collect(), "hybrid")
                        }
                    }
                };
                mode_used = pmode.to_string();

                for mut r in page_results {
                    let id = r.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let score = r.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);

                    let mut page_type_str = "".to_string();
                    let mut page_type_rank: u8 = 0;
                    let mut centrality: usize = 0;
                    if let Some(&idx) = id_index.get(&id) {
                        let meta = &graph[idx];
                        page_type_str = format!("{:?}", meta.page_type).to_lowercase();
                        centrality = meta.relates_to.len();
                        // Compute page_type_rank for sorting
                        page_type_rank = match meta.page_type {
                            crate::engine::PageType::Task => 7,
                            crate::engine::PageType::Spec => 6,
                            crate::engine::PageType::Pattern => 5,
                            crate::engine::PageType::Concept => 4,
                            crate::engine::PageType::Decision => 3,
                            crate::engine::PageType::Howto => 2,
                            crate::engine::PageType::Reference => 1,
                        };
                    }

                    let mut final_score = score;
                    if page_type_str == "task" {
                        // Compute actual days since update from page metadata
                        let days_since = if let Some(&idx) = id_index.get(&id) {
                            let meta = &graph[idx];
                            use chrono::NaiveDate;
                            if let Ok(d) = NaiveDate::parse_from_str(&meta.updated_at, "%Y-%m-%d") {
                                let updated = d.and_hms_opt(0, 0, 0)
                                    .map(|dt| dt.and_utc())
                                    .unwrap_or_else(chrono::Utc::now);
                                let duration = chrono::Utc::now().signed_duration_since(updated);
                                (duration.num_hours() as f64 / 24.0).max(0.0)
                            } else {
                                7.0 // fallback if date unparseable
                            }
                        } else {
                            7.0
                        };
                        let recency = crate::search::recency_boost(days_since, &recency_model, recency_stability);
                        final_score *= recency;
                    }

                    r["score"] = serde_json::json!(final_score);
                    r["type"] = serde_json::json!("page");
                    r["page_type"] = serde_json::json!(page_type_str);
                    r["page_type_rank"] = serde_json::json!(page_type_rank);
                    r["centrality"] = serde_json::json!(centrality);
                    all_results.push(r);
                }
            }

            // 2. Search memory
            if search_memory {
                let mem_index = e.memory_index.load();
                let mem_results: Vec<serde_json::Value> = if mem_index.total_docs > 0 {
                    match mode {
                        crate::embed::SearchMode::Keyword | crate::embed::SearchMode::Hybrid => {
                            let r = mem_index.search(&query, limit);
                            r.into_iter().map(|r| {
                                serde_json::json!({
                                    "id": r.id, "score": r.score, "snippet": r.snippet,
                                    "type": "memory",
                                    "page_type": "memory",
                                    "page_type_rank": 0,
                                    "centrality": 0,
                                })
                            }).collect()
                        }
                        _ => Vec::new(),
                    }
                } else {
                    Vec::new()
                };

                for mut r in mem_results {
                    let score = r.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    // Salience boost per spec FR-7: boost = min(salience_boost, clamp / score)
                    let boost = if score > 0.0 {
                        memory_salience_boost.min(memory_salience_clamp / score)
                    } else {
                        1.0
                    };
                    r["score"] = serde_json::json!(score * boost);
                    all_results.push(r);
                }
            }

            // Merge via RRF if both index searched
            if search_pages && search_memory && all_results.len() > 1 {
                all_results = merge_by_rrf(all_results, rrf_k, limit);
            }

            if !(search_pages && search_memory) {
                all_results.sort_by(|a, b| {
                    b.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0)
                        .partial_cmp(&a.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                all_results.truncate(limit);
            }

            let elapsed = start.elapsed().as_millis() as i64;
            Ok(serde_json::json!({
                "query": query,
                "mode": mode_used,
                "embedder_loaded": embedder_loaded,
                "search_time_ms": elapsed,
                "results": all_results,
                "total": all_results.len(),
            }))
        })
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_search.retrieve",
        "Context assembly with token budget (type: all/page/memory)",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let query = args.require_string("q")?;
            let token_budget = args.optional_int("token_budget").unwrap_or(8192);
            let r#type = args.optional_string("type").unwrap_or_else(|| "all".into());

            let search_pages = r#type == "all" || r#type == "page" || r#type == "task";
            let search_memory = r#type == "all" || r#type == "memory";

            let mut context_text = String::new();
            let mut results: Vec<serde_json::Value> = Vec::new();

            // Page context via graph BFS
            if search_pages {
                let snapshot = e.graph.load();
                let graph = &snapshot.0;
                let index = &snapshot.1;

                let page_ctx = crate::search::retrieve_context(graph, index, &query, token_budget);
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

            // Memory context via flat text
            if search_memory {
                let mem_index = e.memory_index.load();
                if mem_index.total_docs > 0 {
                    let mem_results = mem_index.search(&query, 10);
                    for r in &mem_results {
                        if let Some(sep) = r.id.find(':') {
                            let mem_id = &r.id[sep+1..];
                            // Prevent path traversal
                            if mem_id.contains("..") || mem_id.contains('/') || mem_id.contains('\\') {
                                continue;
                            }
                            // Resolve memory dir from project root
                            let root = e.project_root.read().map_err(|_| ToolError::internal("project_root lock poisoned"))?;
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

    // ─── Search Resolve ────────────────────────────────────────

    let e = engine.clone();
    registry.register_with_desc(
        "wm_search.resolve",
        "Resolve a query to a page ID",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let q = args.require_string("q")?;

            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let index = &snapshot.1;

            // Try exact ID match first
            if let Some(&idx) = index.get(&q) {
                let meta = &graph[idx];
                return Ok(serde_json::json!({
                    "resolved": true,
                    "id": meta.id,
                    "title": meta.title,
                    "page_type": format!("{:?}", meta.page_type).to_lowercase(),
                }));
            }

            // Try title match
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

            // Try BM25 search
            let bm25 = e.bm25_index.load();
            let results = bm25.search(&q, 5);
            if !results.is_empty() {
                let candidates: Vec<serde_json::Value> = results.iter().map(|r| {
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

    // ─── Page Tools ───────────────────────────────────────────

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
                // Infer from path: "wiki/tasks/auth" → "task"
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
            // Submit debounced rebuild, replacing any pending
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

            // Find file from graph
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

    // ─── Page Link / Unlink ────────────────────────────────────

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

            // Remove the specific relates_to entry by re-writing without it
            let update = serde_json::json!({
                "remove_relates_to": target
            });
            page::update_page(&e, &id, &update)?;
            Ok(serde_json::json!({ "id": id, "target": target, "status": "unlinked" }))
        }),
    );

    // ─── Source Tools ─────────────────────────────────────────

    let e = engine.clone();
    registry.register_with_desc(
        "wm_source.add",
        "Add a raw source file to the registry",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let path = args.require_string("path")?;
            let id = source::add_source(&e, &path)?;
            Ok(serde_json::json!({ "id": id, "state": "pending" }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_source.process",
        "Process a source (pending→processing)",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let content = source::process_source(&e, &id)?;
            Ok(serde_json::json!({ "id": id, "content": content }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_source.complete",
        "Complete source processing (processing→done)",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let refs = args.optional_string_array("page_refs");
            source::complete_source(&e, &id, &refs)?;
            Ok(serde_json::json!({ "id": id, "status": "done", "pages": refs.len() }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_source.error",
        "Mark a source as errored",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let msg = args
                .optional_string("message")
                .unwrap_or_else(|| "Unknown error".to_string());
            source::error_source(&e, &id, &msg)?;
            Ok(serde_json::json!({ "id": id, "status": "error", "message": msg }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_source.list",
        "List sources with optional state filter",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let state = args.optional_string("state");
            let sources = source::list_sources(&e, state.as_deref())?;
            Ok(serde_json::json!({ "sources": sources, "total": sources.len() }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_source.verify",
        "Verify source staleness by hash",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let is_stale = source::verify_source(&e, &id)?;
            Ok(serde_json::json!({ "id": id, "stale": is_stale }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_source.discover",
        "Scan configured directories for new sources",
        Arc::new(move |_params| {
            let (dirs, exts) = {
                let config = e.config.read().map_err(|_| ToolError::internal("config lock poisoned"))?;
                (config.source_dirs.clone(), config.source_extensions.clone())
            };
            let discovered = source::discover_sources(&e, &dirs, Some(&exts))?;
            Ok(serde_json::json!({ "discovered": discovered, "total": discovered.len() }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_source.remove",
        "Remove a source from the registry",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            source::remove_source(&e, &id)?;
            Ok(serde_json::json!({ "id": id, "status": "removed" }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_source.status",
        "Get detailed source status",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let status = source::source_status(&e, &id)?;
            Ok(status)
        }),
    );

    // ─── Graph Tools ──────────────────────────────────────────

    let e = engine.clone();
    registry.register_with_desc(
        "wm_graph.neighbors",
        "Get typed edges from a page",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let query = args.optional_string("query");

            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let index = &snapshot.1;

            let start = index
                .get(&id)
                .ok_or_else(|| ToolError::not_found("page", &id))?;
            let mut neighbors = Vec::new();

            for edge in graph.edges(*start) {
                let target = edge.target();
                let edge_type = edge.weight();
                let meta = &graph[target];

                let score = if let Some(ref q) = query {
                    let q_lower = q.to_lowercase();
                    let title_match = if meta.title.to_lowercase().contains(&q_lower) {
                        4.0
                    } else {
                        0.0
                    };
                    let tag_match = if meta
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&q_lower))
                    {
                        2.2
                    } else {
                        0.0
                    };
                    let exact_title = if meta.title.to_lowercase() == q_lower {
                        8.0
                    } else {
                        0.0
                    };
                    edge_type.priority() as f64 * (1.0 + title_match + tag_match + exact_title)
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
        }),
    );

    // ─── Graph Stats ──────────────────────────────────────────

    let e = engine.clone();
    registry.register_with_desc(
        "wm_graph.stats",
        "Graph statistics (node/edge counts by type)",
        Arc::new(move |_params| {
            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let mut type_counts: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();
            for idx in graph.node_indices() {
                let type_name = format!("{:?}", graph[idx].page_type).to_lowercase();
                *type_counts.entry(type_name).or_insert(0) += 1;
            }
            Ok(serde_json::json!({
                "nodes": graph.node_count(),
                "edges": graph.edge_count(),
                "types": type_counts,
            }))
        }),
    );

    // ─── Graph Subgraph ───────────────────────────────────────

    let e = engine.clone();
    registry.register_with_desc(
        "wm_graph.subgraph",
        "Get neighborhood around a page node",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let center = args.require_string("center")?;
            let depth = args.optional_int("depth").unwrap_or(1).min(5);

            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let index = &snapshot.1;

            let start = match index.get(&center) {
                Some(s) => *s,
                None => return Err(ToolError::not_found("page", &center)),
            };

            // BFS to collect nodes within `depth` hops
            use std::collections::VecDeque;
            let mut visited = std::collections::HashSet::new();
            let mut queue = VecDeque::new();
            let mut nodes = Vec::new();
            let mut edges = Vec::new();

            visited.insert(start);
            queue.push_back((start, 0usize));

            while let Some((current, d)) = queue.pop_front() {
                if d > depth {
                    continue;
                }
                let meta = &graph[current];
                nodes.push(serde_json::json!({
                    "id": meta.id, "title": meta.title,
                    "type": format!("{:?}", meta.page_type).to_lowercase(),
                    "depth": d,
                }));
                for edge in graph.edges(current) {
                    let target = edge.target();
                    edges.push(serde_json::json!({
                        "source": graph[current].id,
                        "target": graph[target].id,
                        "type": format!("{:?}", edge.weight()).to_lowercase(),
                    }));
                    if visited.insert(target) {
                        queue.push_back((target, d + 1));
                    }
                }
            }

            Ok(serde_json::json!({
                "center": center,
                "depth": depth,
                "nodes": nodes,
                "edges": edges,
                "node_count": nodes.len(),
            }))
        }),
    );

    // ─── Graph Path (simple BFS) ──────────────────────────────

    let e = engine.clone();
    registry.register_with_desc(
        "wm_graph.path",
        "Find shortest path between two pages",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let start_id = args.require_string("start")?;
            let end_id = args.require_string("end")?;
            let max_depth = args.optional_int("max_depth").unwrap_or(10);

            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let index = &snapshot.1;

            let start = index
                .get(&start_id)
                .ok_or_else(|| ToolError::not_found("page", &start_id))?;
            let end = index
                .get(&end_id)
                .ok_or_else(|| ToolError::not_found("page", &end_id))?;

            // BFS with depth tracking
            use std::collections::VecDeque;
            let mut visited = std::collections::HashSet::new();
            let mut queue = VecDeque::new();
            // parent[node] = (parent_node, edge_type, depth)
            let mut parent: std::collections::HashMap<
                petgraph::stable_graph::NodeIndex,
                (petgraph::stable_graph::NodeIndex, String, usize),
            > = std::collections::HashMap::new();

            visited.insert(*start);
            queue.push_back((*start, 0usize));
            let mut found = false;

            while let Some((current, depth)) = queue.pop_front() {
                if current == *end {
                    found = true;
                    break;
                }

                if depth >= max_depth {
                    continue;
                }

                for edge in graph.edges(current) {
                    let target = edge.target();
                    if visited.insert(target) {
                        let edge_type = format!("{:?}", edge.weight()).to_lowercase();
                        parent.insert(target, (current, edge_type, depth + 1));
                        queue.push_back((target, depth + 1));
                    }
                }
            }

            if found {
                // Reconstruct path
                let mut path = Vec::new();
                let mut current = *end;
                while current != *start {
                    if let Some((prev, edge_type, _depth)) = parent.get(&current) {
                        path.push(serde_json::json!({
                            "id": graph[current].id.clone(),
                            "title": graph[current].title.clone(),
                            "edge_from_parent": edge_type,
                        }));
                        current = *prev;
                    } else {
                        break;
                    }
                }
                path.push(serde_json::json!({
                    "id": graph[*start].id.clone(),
                    "title": graph[*start].title.clone(),
                    "edge_from_parent": null,
                }));
                path.reverse();

                Ok(serde_json::json!({ "path": path, "length": path.len() }))
            } else {
                Ok(serde_json::json!({ "path": [], "length": 0, "note": "No path found" }))
            }
        }),
    );

    // ─── Lint Tools ───────────────────────────────────────────

    let e = engine.clone();
    registry.register_with_desc("wm_lint.check", "Check wiki for common issues", Arc::new(move |_params| {
        let snapshot = e.graph.load();
        let graph = &snapshot.0;
        let index = &snapshot.1;
        let mut issues = Vec::new();
        let mut cycle_info = None;

        for idx in graph.node_indices() {
            let meta = &graph[idx];

            // Check for orphan pages
            let has_inbound = graph.edges_directed(idx, petgraph::Direction::Incoming).count() > 0;
            if !has_inbound {
                issues.push(serde_json::json!({
                    "type": "orphan",
                    "severity": "warning",
                    "id": meta.id,
                    "message": "No inbound edges — consider adding relationships"
                }));
            }

            // Check for broken relates_to refs
            for edge in graph.edges(idx) {
                let target = edge.target();
                let target_id = &graph[target].id;
                if !index.contains_key(target_id) {
                    issues.push(serde_json::json!({
                        "type": "broken_ref",
                        "severity": "error",
                        "id": meta.id,
                        "message": format!("References '{}' which doesn't exist in the graph", target_id)
                    }));
                }
            }

            // Check for missing ACs on task pages
            if meta.page_type == crate::engine::PageType::Task && meta.acceptance_criteria.is_empty() {
                issues.push(serde_json::json!({
                    "type": "missing_acs",
                    "severity": "warning",
                    "id": meta.id,
                    "message": "Task has no acceptance criteria"
                }));
            }

            // Check for missing status on spec pages
            if meta.page_type == crate::engine::PageType::Spec {
                let is_draft = meta.status == crate::engine::PageStatus::Draft;
                if is_draft {
                    issues.push(serde_json::json!({
                        "type": "spec_status",
                        "severity": "info",
                        "id": meta.id,
                        "message": "Spec status is draft — consider setting reviewed/approved"
                    }));
                }
            }
        }

        // Check for stale sources — verify content hash for each source
        let registry = e.source_registry.read().map_err(|_| ToolError::internal("registry lock poisoned"))?;
        let mut stale_count = 0usize;
        for entry in registry.values() {
            let is_stale = if entry.state == crate::engine::SourceState::Stale {
                true
            } else if entry.stored_path.exists() {
                // Compute current content hash and compare
                let content = match std::fs::read(&entry.stored_path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                let current_hash = Sha256::digest(&content);
                let current_hex = hex::encode(current_hash);
                current_hex != entry.content_hash
            } else {
                false
            };

            if is_stale {
                stale_count += 1;
                issues.push(serde_json::json!({
                    "type": "stale_source",
                    "severity": "warning",
                    "id": entry.id,
                    "message": format!("Source '{}' content hash changed — needs reprocessing", entry.id),
                }));
            }
        }
        if stale_count > 0 {
            issues.push(serde_json::json!({
                "type": "stale_sources_summary",
                "severity": "info",
                "id": "registry",
                "message": format!("{} stale source(s) found. Run source.verify to check.", stale_count),
            }));
        }

        // Cycle detection report
        if petgraph::algo::is_cyclic_directed(graph) {
            cycle_info = Some("Cycle detected in graph — BFS uses visited tracking to prevent infinite loops");
            issues.push(serde_json::json!({
                "type": "cycle",
                "severity": "warning",
                "id": "graph",
                "message": "Cycle detected in wiki graph"
            }));
        }

        Ok(serde_json::json!({
            "issues": issues,
            "total": issues.len(),
            "has_cycles": cycle_info.is_some(),
            "stale_sources": stale_count,
        }))
    }));

    // ─── Lint Fix (auto-fix common issues) ─────────────────────

    let e = engine.clone();
    registry.register_with_desc(
        "wm_lint.fix",
        "Auto-fix common issues",
        Arc::new(move |_params| {
            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let fixed = crate::graph::lint_fix(graph, &e.write_channel);

            if fixed > 0 {
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
            }

            Ok(serde_json::json!({
                "fixed": fixed,
                "message": format!("Fixed {} issue(s)", fixed),
            }))
        }),
    );

    // ─── Validate Tools ───────────────────────────────────────

    let e = engine.clone();
    registry.register_with_desc(
        "wm_validate.check",
        "Validate wiki health",
        Arc::new(move |_params| {
            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let mut errors: Vec<serde_json::Value> = Vec::new();

            for idx in graph.node_indices() {
                let meta = &graph[idx];

                // All pages need title
                if meta.title.is_empty() {
                    errors.push(serde_json::json!({
                        "id": meta.id, "field": "title", "message": "Title is required"
                    }));
                }

                // Per-type checks
                match meta.page_type {
                    crate::engine::PageType::Task => {
                        if meta.acceptance_criteria.is_empty() {
                            errors.push(serde_json::json!({
                                "id": meta.id, "field": "acceptance_criteria",
                                "message": "Task should have at least one acceptance criterion"
                            }));
                        }
                        if meta.assignee.is_none() {
                            errors.push(serde_json::json!({
                                "id": meta.id, "field": "assignee",
                                "message": "Task should have an assignee"
                            }));
                        }
                    }
                    crate::engine::PageType::Spec => {
                        if meta.status == crate::engine::PageStatus::Draft
                            && meta.stakeholders.is_empty()
                        {
                            errors.push(serde_json::json!({
                                "id": meta.id, "field": "stakeholders",
                                "message": "Spec should have stakeholders defined"
                            }));
                        }
                    }
                    crate::engine::PageType::Decision => {
                        if meta.decision.is_none() {
                            errors.push(serde_json::json!({
                                "id": meta.id, "field": "decision",
                                "message": "Decision page should have context, options, rationale"
                            }));
                        }
                    }
                    crate::engine::PageType::Pattern => {
                        if meta.pattern.is_none() {
                            errors.push(serde_json::json!({
                                "id": meta.id, "field": "pattern",
                                "message": "Pattern page should have when_to_use and example"
                            }));
                        }
                    }
                    _ => {}
                }
            }

            Ok(serde_json::json!({
                "status": if errors.is_empty() { "pass" } else { "fail" },
                "nodes": graph.node_count(),
                "edges": graph.edge_count(),
                "errors": errors,
                "total_errors": errors.len(),
            }))
        }),
    );

    // ─── Index Tools ─────────────────────────────────────────

    let e = engine.clone();
    registry.register_with_desc(
        "wm_index.rebuild",
        "Full rebuild (graph + BM25 + embeddings)",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let skip_embed = args.optional_bool("skip_embed");
            let embed_batch_size = args.optional_int("embed_batch_size").unwrap_or(32);
            let root = std::env::current_dir().map_err(|e| ToolError::internal(e.to_string()))?;
            let wiki_dir = root.join(".wm").join("wiki");

            if !wiki_dir.exists() {
                return Err(ToolError::internal(
                    "No wiki directory found. Run 'wm init' first.",
                ));
            }

            // 1. Rebuild graph
            let count = e.rebuild_graph(&wiki_dir);

            // 2. Rebuild section corpus
            let sections = crate::graph::build_sections_from_wiki(&wiki_dir);
            e.section_corpus.store(Arc::new(sections.clone()));

            // 3. Rebuild BM25 index
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
            let bm25 = crate::search::Bm25Index::build(docs);
            e.bm25_index.store(Arc::new(bm25));

            // 2.5. Rebuild memory index
            let memory_dir = root.join(".wm").join("memory");
            let mem_count = e.rebuild_memory_index(&memory_dir);

            // 4. Build embeddings (if embedder loaded and not skipped)
            let embed_count = if e.embedder.is_loaded() && !skip_embed {
                let old_hashes = e.vector_store.hashes.load_full();
                let old_entries = e.vector_store.entries.load_full();
                match crate::embed::build_embeddings(
                    &*e.embedder,
                    &sections,
                    &old_hashes,
                    Some(&old_entries),
                    embed_batch_size,
                ) {
                    Ok((new_entries, new_hashes)) => {
                        e.vector_store.swap(new_entries.clone(), new_hashes);
                        // Persist to disk
                        let root = std::env::current_dir().unwrap_or_default();
                        let vectors_path = root.join(".wm").join("state").join("vectors.bin");
                        if let Err(err) = e.vector_store.save_to_disk(&vectors_path) {
                            tracing::warn!("Failed to persist vectors.bin: {}", err);
                        }
                        new_entries.len()
                    }
                    Err(err) => {
                        tracing::warn!("Embedding rebuild failed: {}", err);
                        0
                    }
                }
            } else if !e.embedder.is_loaded() && !skip_embed {
                tracing::info!("Skipping embeddings — no model loaded. Run 'wm model download'.");
                0
            } else {
                0
            };

            // 5. Auto-generate index.md
            let _ = crate::graph::auto_generate_index(&wiki_dir, &e.graph.load().0);

            e.stale_flag
                .store(false, std::sync::atomic::Ordering::Release);

            Ok(serde_json::json!({
                "status": "ok",
                "graph_nodes": count,
                "sections": sections.len(),
                "sections_embedded": embed_count,
                "memory_indexed": mem_count,
                "message": "Full rebuild complete"
            }))
        }),
    );

    // ─── Index Embed (standalone) ─────────────────────────────

    let e = engine.clone();
    registry.register_with_desc(
        "wm_index.embed",
        "Build embedding vectors only",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let batch_size = args.optional_int("batch_size").unwrap_or(32);
            let _force = args.optional_bool("force");

            if !e.embedder.is_loaded() {
                return Err(ToolError::internal(
                    "No embedding model loaded. Run 'wm model download' first.",
                ));
            }

            let sections = e.section_corpus.load();
            if sections.is_empty() {
                return Err(ToolError::internal(
                    "No sections found. Run 'wm index.rebuild' first.",
                ));
            }

            let old_hashes = e.vector_store.hashes.load_full();
            let old_entries = e.vector_store.entries.load_full();

            match crate::embed::build_embeddings(
                &*e.embedder,
                &sections,
                &old_hashes,
                Some(&old_entries),
                batch_size,
            ) {
                Ok((new_entries, new_hashes)) => {
                    e.vector_store.swap(new_entries.clone(), new_hashes);
                    let root = std::env::current_dir().unwrap_or_default();
                    let vectors_path = root.join(".wm").join("state").join("vectors.bin");
                    if let Err(err) = e.vector_store.save_to_disk(&vectors_path) {
                        tracing::warn!("Failed to persist vectors.bin: {}", err);
                    }
                    Ok(serde_json::json!({
                        "status": "ok",
                        "sections_embedded": new_entries.len(),
                        "message": "Embedding complete"
                    }))
                }
                Err(err) => Err(ToolError::internal(format!("Embedding failed: {}", err))),
            }
        }),
    );

    // ─── Index Status ─────────────────────────────────────────

    let e = engine.clone();
    registry.register_with_desc(
        "wm_index.status",
        "Show index state (sections, vectors, stale)",
        Arc::new(move |_params| {
            let (graph_nodes, graph_edges) = {
                let snap = e.graph.load();
                (snap.0.node_count(), snap.0.edge_count())
            };
            let sections = e.section_corpus.load().len();
            let bm25_docs = e.bm25_index.load().total_docs;
            let memory_docs = e.memory_index.load().total_docs;
            let vectors = e.vector_store.snapshot().len();
            let model = e.embedder.model_name().to_string();
            let embedder_loaded = e.embedder.is_loaded();
            let stale = e.stale_flag.load(std::sync::atomic::Ordering::Acquire);

            Ok(serde_json::json!({
                "graph_nodes": graph_nodes,
                "graph_edges": graph_edges,
                "sections": sections,
                "bm25_indexed": bm25_docs,
                "memory_indexed": memory_docs,
                "vectors_persisted": vectors,
                "model": model,
                "embedder_loaded": embedder_loaded,
                "stale": stale,
            }))
        }),
    );

    // ─── Task Tools ───────────────────────────────────────────

    let e = engine.clone();
    registry.register_with_desc(
        "wm_task.check_ac",
        "Check an acceptance criterion",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let ac_indices = args.optional_string_array("criteria");
            let indices: Vec<u64> = ac_indices.iter().filter_map(|s| s.parse().ok()).collect();
            let update = serde_json::json!({ "checked_ac": indices });
            page::update_page(&e, &id, &update)?;
            Ok(serde_json::json!({ "id": id, "checked": indices }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_task.uncheck_ac",
        "Uncheck an acceptance criterion",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let ac_indices = args.optional_string_array("criteria");
            let indices: Vec<u64> = ac_indices.iter().filter_map(|s| s.parse().ok()).collect();
            let update = serde_json::json!({ "unchecked_ac": indices });
            page::update_page(&e, &id, &update)?;
            Ok(serde_json::json!({ "id": id, "unchecked": indices }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc("wm_task.board", "Task board grouped by status", Arc::new(move |_params| {
        let snapshot = e.graph.load();
        let graph = &snapshot.0;
        let mut todo = Vec::new();
        let mut in_progress = Vec::new();
        let mut done = Vec::new();
        let mut blocked = Vec::new();

        for idx in graph.node_indices() {
            let meta = &graph[idx];
            if meta.page_type != crate::engine::PageType::Task { continue; }
            let entry = serde_json::json!({
                "id": meta.id,
                "title": meta.title,
                "priority": format!("{:?}", meta.priority.as_ref().unwrap_or(&crate::engine::Priority::Medium)).to_lowercase(),
            });
            match meta.status {
                crate::engine::PageStatus::Todo => todo.push(entry),
                crate::engine::PageStatus::InProgress => in_progress.push(entry),
                crate::engine::PageStatus::Done => done.push(entry),
                crate::engine::PageStatus::Blocked => blocked.push(entry),
                _ => todo.push(entry),
            }
        }

        Ok(serde_json::json!({
            "columns": {
                "todo": todo,
                "in_progress": in_progress,
                "done": done,
                "blocked": blocked,
            },
            "counts": {
                "todo": todo.len(),
                "in_progress": in_progress.len(),
                "done": done.len(),
                "blocked": blocked.len(),
            },
        }))
    }));

    // ─── Log Tools ────────────────────────────────────────────

    registry.register_with_desc(
        "wm_log.recent",
        "Recent log entries",
        Arc::new(|params| {
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
        }),
    );

    registry.register_with_desc(
        "wm_log.since",
        "Log entries since a marker",
        Arc::new(|params| {
            let args = ToolArgs::new(params);
            let marker = args.require_string("marker")?;
            let log_path = std::path::Path::new(".wm").join("wiki").join("log.md");
            let content = std::fs::read_to_string(&log_path).unwrap_or_default();
            let lines: Vec<&str> = content
                .lines()
                .skip_while(|line| !line.contains(&marker))
                .skip(1) // skip the marker line itself
                .collect();
            Ok(serde_json::json!({
                "entries": lines,
                "total": lines.len(),
            }))
        }),
    );

    registry.register_with_desc(
        "wm_log.filter",
        "Filter log entries by text",
        Arc::new(|params| {
            let args = ToolArgs::new(params);
            let text = args.require_string("text")?;
            let log_path = std::path::Path::new(".wm").join("wiki").join("log.md");
            let content = std::fs::read_to_string(&log_path).unwrap_or_default();
            let lines: Vec<&str> = content
                .lines()
                .filter(|line| line.to_lowercase().contains(&text.to_lowercase()))
                .collect();
            Ok(serde_json::json!({
                "entries": lines,
                "total": lines.len(),
            }))
        }),
    );

    // ─── Model Tools ──────────────────────────────────────────

    let e = engine.clone();
    registry.register_with_desc(
        "wm_model.list",
        "List cached and available models",
        Arc::new(move |_params| {
            let model_name = e.embedder.model_name().to_string();
            let loaded = e.embedder.is_loaded();
            let indexed = e.vector_store.snapshot().len();

            // Check cached models
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".into());
            let models_dir = std::path::PathBuf::from(home).join(".wm").join("models");
            let mut cached_models = Vec::new();
            if models_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&models_dir) {
                    for entry in entries.flatten() {
                        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                            let name = entry.file_name().to_string_lossy().to_string();
                            cached_models.push(serde_json::json!({
                                "name": name,
                                "cached": true,
                                "active": name == model_name && loaded,
                            }));
                        }
                    }
                }
            }

            Ok(serde_json::json!({
                "models": cached_models,
                "active_model": model_name,
                "loaded": loaded,
                "sections_indexed": indexed,
                "available_remote": [
                    {"name": "bge-small-en-v1.5", "dim": 384, "size_mb": 134},
                    {"name": "bge-base-en-v1.5", "dim": 768, "size_mb": 438},
                    {"name": "all-MiniLM-L6-v2", "dim": 384, "size_mb": 90},
                ],
            }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_model.status",
        "Show current model state",
        Arc::new(move |_params| {
            Ok(serde_json::json!({
                "model": e.embedder.model_name(),
                "loaded": e.embedder.is_loaded(),
                "dimensions": e.embedder.output_dim(),
                "sections_indexed": e.vector_store.snapshot().len(),
            }))
        }),
    );

    registry.register_with_desc(
        "wm_model.download",
        "Download an embedding model",
        Arc::new(|params| {
            let args = ToolArgs::new(params);
            #[cfg(feature = "embed")]
            {
                let name = args.require_string("name")?;
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".into());
                let models_dir = std::path::PathBuf::from(home).join(".wm").join("models");
                match crate::onnx::download_model(&name, &models_dir) {
                    Ok(dir) => Ok(serde_json::json!({
                        "status": "ok",
                        "message": format!("Model downloaded to {}", dir.display()),
                        "model_name": name,
                    })),
                    Err(e) => Err(ToolError::internal(format!("Download failed: {}", e))),
                }
            }

            #[cfg(not(feature = "embed"))]
            {
                let _name = args.require_string("name")?;
                Err(ToolError::internal(
                    "Model download requires the 'embed' feature. Rebuild with --features embed.",
                ))
            }
        }),
    );

    registry.register_with_desc(
        "wm_model.remove",
        "Remove a cached model",
        Arc::new(|params| {
            let args = ToolArgs::new(params);
            let name = args.require_string("name")?;

            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".into());
            let model_dir = std::path::PathBuf::from(home)
                .join(".wm")
                .join("models")
                .join(&name);

            if model_dir.exists() {
                std::fs::remove_dir_all(&model_dir)
                    .map_err(|e| ToolError::internal(format!("Failed to remove model: {}", e)))?;
            }

            Ok(serde_json::json!({
                "status": "removed",
                "model_name": name,
            }))
        }),
    );

    // ─── Time Tracking Tools ───────────────────────────────────

    let e = engine.clone();
    registry.register_with_desc(
        "wm_time.start",
        "Start time tracking on a task",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let now = chrono::Utc::now().to_rfc3339();
            let update = serde_json::json!({ "time_started": now });
            page::update_page(&e, &id, &update)?;
            Ok(serde_json::json!({ "id": id, "time_started": now, "status": "started" }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_time.stop",
        "Stop time tracking, record elapsed",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;

            // Read current page to get time_started and existing time_spent
            let snapshot = e.graph.load();
            let index = &snapshot.1;
            let node_idx = index
                .get(&id)
                .ok_or_else(|| crate::error::ToolError::not_found("page", &id))?;
            let meta = &snapshot.0[*node_idx];
            let file_path = &meta.path;

            let content = std::fs::read_to_string(file_path)
                .map_err(|e| crate::error::ToolError::internal(format!("read error: {}", e)))?;
            let (fm, _) = crate::parser::extract_frontmatter(&content);

            let time_started = fm
                .as_ref()
                .and_then(|f| f.time_started.as_deref())
                .unwrap_or("");

            // Compute elapsed time since time_started
            let now = chrono::Utc::now();
            let elapsed_minutes = if let Ok(started) = chrono::DateTime::parse_from_rfc3339(time_started) {
                let dur = now.signed_duration_since(started);
                (dur.num_hours() * 60 + dur.num_minutes() % 60) as f64
            } else {
                0.0
            };

            // Read existing time_spent and accumulate
            let existing_spent = fm.as_ref().and_then(|f| f.time_spent.as_deref()).unwrap_or("");
            let existing_minutes = parse_duration_to_minutes(existing_spent);
            let total_minutes = existing_minutes + elapsed_minutes;
            let total_hours = (total_minutes / 60.0).floor() as i64;
            let total_mins = (total_minutes % 60.0) as i64;
            let total = format!("{}h {}m", total_hours, total_mins);

            let update = serde_json::json!({
                "time_spent": total,
                "time_started": serde_json::Value::Null,
            });
            page::update_page(&e, &id, &update)?;
            Ok(serde_json::json!({ "id": id, "time_spent": total, "status": "stopped" }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_time.add",
        "Manually add time to a task",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let duration = args.require_string("duration")?;

            // Read existing time_spent from frontmatter
            let snapshot = e.graph.load();
            let index = &snapshot.1;
            let node_idx = index
                .get(&id)
                .ok_or_else(|| crate::error::ToolError::not_found("page", &id))?;
            let meta = &snapshot.0[*node_idx];
            let file_path = &meta.path;

            let content = std::fs::read_to_string(file_path)
                .map_err(|e| crate::error::ToolError::internal(format!("read error: {}", e)))?;
            let (fm, _) = crate::parser::extract_frontmatter(&content);

            let existing_spent = fm.as_ref().and_then(|f| f.time_spent.as_deref()).unwrap_or("");
            let existing_minutes = parse_duration_to_minutes(existing_spent);
            let added_minutes = parse_duration_to_minutes(&duration);
            let total_minutes = existing_minutes + added_minutes;
            let total_hours = (total_minutes / 60.0).floor() as i64;
            let total_mins = (total_minutes % 60.0) as i64;
            let total = format!("{}h {}m", total_hours, total_mins);

            let update = serde_json::json!({ "time_spent": total });
            page::update_page(&e, &id, &update)?;
            Ok(serde_json::json!({ "id": id, "time_spent": total, "status": "added" }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_time.report",
        "Time report across all tasks",
        Arc::new(move |_params| {
            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let mut tasks: Vec<serde_json::Value> = Vec::new();
            let mut total_hours = 0f64;
            let mut total_estimated_hours = 0f64;

            for idx in graph.node_indices() {
                let meta = &graph[idx];
                if meta.page_type != crate::engine::PageType::Task {
                    continue;
                }

                // Read frontmatter for time_started/time_spent/estimate
                let file_path = &meta.path;
                if !file_path.exists() {
                    continue;
                }
                let content = std::fs::read_to_string(file_path).unwrap_or_default();
                let (fm, _) = crate::parser::extract_frontmatter(&content);

                let time_spent = fm
                    .as_ref()
                    .and_then(|f| f.time_spent.as_deref())
                    .unwrap_or("");
                let time_started = fm
                    .as_ref()
                    .and_then(|f| f.time_started.as_deref())
                    .unwrap_or("");
                let estimate = fm.as_ref().and_then(|f| f.estimate);

                // Parse hours from time_spent (format: "Xh Ym")
                if let Some(h) = time_spent
                    .split('h')
                    .next()
                    .and_then(|s| s.trim().parse::<f64>().ok())
                {
                    total_hours += h;
                }

                if let Some(est) = estimate {
                    total_estimated_hours += est as f64;
                }

                if !time_spent.is_empty() || !time_started.is_empty() || estimate.is_some() {
                    tasks.push(serde_json::json!({
                        "id": meta.id,
                        "title": meta.title,
                        "time_spent": time_spent,
                        "time_started": time_started,
                        "estimate": estimate,
                    }));
                }
            }

            Ok(serde_json::json!({
                "tasks": tasks,
                "total_tasks": tasks.len(),
                "total_hours": total_hours,
                "total_estimated_hours": total_estimated_hours,
            }))
        }),
    );

    // ─── Project Tools ────────────────────────────────────────

    registry.register_with_desc(
        "wm_project.status",
        "Project status information",
        Arc::new(|_params| {
            let root = std::env::current_dir().ok();
            let project = root.as_ref().and_then(|r| {
                let config_path = r.join(".wm").join("config.json");
                std::fs::read_to_string(config_path).ok()
            });
            Ok(serde_json::json!({
                "project": if project.is_some() { "active" } else { "none" },
                "root": root.map(|r| r.to_string_lossy().to_string()),
            }))
        }),
    );

    registry.register_with_desc(
        "wm_project.detect",
        "Detect project root from current directory",
        Arc::new(|_params| {
            let root = crate::config::detect_project_root();
            match root {
            Some(path) => Ok(serde_json::json!({
                "project": "detected",
                "root": path.to_string_lossy().to_string(),
            })),
            None => Err(crate::error::ToolError::not_found("project",
                "No .wm/config.json found in current or parent directories. Run 'wm init' first.")),
        }
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_project.set",
        "Set the current project root",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let path = args.require_string("path")?;
            let root = std::path::PathBuf::from(&path);
            if !root.join(".wm").join("config.json").exists() {
                return Err(crate::error::ToolError::not_found(
                    "project",
                    &format!("No .wm/config.json found at {}", root.display()),
                ));
            }
            e.project_root.write().map_err(|_| ToolError::internal("project_root lock poisoned"))?.clone_from(&root);
            Ok(serde_json::json!({ "project": "set", "root": root.to_string_lossy().to_string() }))
        }),
    );
}

/// Merge results from multiple entity types using RRF.
/// Assigns per-type ranks based on position within each type group, then fuses.
fn merge_by_rrf(results: Vec<serde_json::Value>, k: f64, limit: usize) -> Vec<serde_json::Value> {
    use std::collections::HashMap;
    // Partition by type first
    let mut by_type: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
    for r in results {
        let t = r.get("type").and_then(|v| v.as_str()).unwrap_or("page").to_string();
        by_type.entry(t).or_default().push(r);
    }
    // Assign per-type ranks and fuse via RRF
    let mut rrf_scores: HashMap<String, (f64, serde_json::Value)> = HashMap::new();
    for (_type, typed_results) in &by_type {
        for (rank, r) in typed_results.iter().enumerate() {
            if let Some(id) = r.get("id").and_then(|v| v.as_str()) {
                let score = 1.0 / (k + rank as f64);
                let entry = rrf_scores.entry(id.to_string()).or_insert((0.0, r.clone()));
                entry.0 += score;
            }
        }
    }
    let mut ranked: Vec<(f64, serde_json::Value)> = rrf_scores.into_values().collect();
    ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(limit);
    ranked.into_iter().map(|(score, mut r)| {
        r["score"] = serde_json::json!(score);
        r
    }).collect()
}
