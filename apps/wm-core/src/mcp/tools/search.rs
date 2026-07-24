use crate::mcp::prelude::*;
use wm_embed::SearchMode;



#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum SearchType {
    All,
    Page,
    Task,
    Memory,
}

impl SearchType {
    fn as_str(&self) -> &'static str {
        match self {
            SearchType::All => "all",
            SearchType::Page => "page",
            SearchType::Task => "task",
            SearchType::Memory => "memory",
        }
    }
}


#[derive(Deserialize, JsonSchema)]
struct WmSearchQueryInput {
    #[schemars(description = "Search query")]
    q: String,
    #[schemars(description = "Search type: all/page/task/memory")]
    r#type: Option<SearchType>,
    #[schemars(description = "Search mode: auto/keyword/semantic/hybrid")]
    mode: Option<SearchMode>,
    #[schemars(description = "Max results")]
    limit: Option<usize>,
    #[schemars(description = "Result offset")]
    offset: Option<usize>,
    #[schemars(description = "Enable recency boost")]
    recency: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
struct WmSearchRetrieveInput {
    #[schemars(description = "Search query")]
    q: String,
    #[schemars(description = "Token budget for context")]
    token_budget: Option<usize>,
    #[schemars(description = "Source type: all/page/memory")]
    r#type: Option<SearchType>,
}

#[derive(Deserialize, JsonSchema)]
struct WmSearchResolveInput {
    #[schemars(description = "Query or page ID to resolve")]
    q: String,
}

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_typed(
        "wm_search.query",
        "Search the wiki and/or memory (keyword/semantic/hybrid)",
        move |input: WmSearchQueryInput| {
            let start = std::time::Instant::now();
            let embedder_loaded = e.embedder.is_loaded();

            let search_type = input.r#type.unwrap_or(SearchType::All).as_str().to_string();
            let search_mode = input.mode.unwrap_or(SearchMode::Auto);

            let params = crate::search::QueryParams {
                query: input.q.trim().to_string(),
                r#type: search_type,
                mode: search_mode.to_string(),
                limit: input.limit.unwrap_or(10),
                offset: input.offset.unwrap_or(0),
                recency: input.recency.unwrap_or(true),
            };

            let resp = crate::search::run_unified_search(&e, &params)
                .map_err(ToolError::internal)?;

            let json_results: Vec<serde_json::Value> = resp.results.into_iter().map(|r| {
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
                "query": input.q,
                "mode": if search_mode == SearchMode::Auto {
                    wm_embed::SearchMode::auto_detect(&input.q).to_string()
                } else {
                    search_mode.to_string()
                },
                "embedder_loaded": embedder_loaded,
                "search_time_ms": elapsed,
                "degraded": resp.degraded,
                "warning": resp.warning,
                "results": json_results,
                "total": json_results.len(),
            }))
        },
    );

    let e = engine.clone();
    registry.register_typed(
        "wm_search.retrieve",
        "Context assembly with token budget (type: all/page/memory)",
        move |input: WmSearchRetrieveInput| {
            let search_type = input.r#type.unwrap_or(SearchType::All);
            let search_pages = matches!(search_type, SearchType::All | SearchType::Page | SearchType::Task);
            let search_memory = matches!(search_type, SearchType::All | SearchType::Memory);

            let mut context_text = String::new();
            let mut results: Vec<serde_json::Value> = Vec::new();

            if search_pages {
                let snapshot = e.graph.load();
                let graph = &snapshot.0;
                let index = &snapshot.1;

                let qp = crate::search::QueryParams {
                    query: input.q.clone(),
                    r#type: "page".into(),
                    mode: "auto".into(),
                    limit: 1,
                    offset: 0,
                    recency: false,
                };
                let resp = crate::search::run_unified_search(&e, &qp).unwrap_or_default();
                let bfs_seed = resp.results
                    .first()
                    .map(|r| r.id.clone())
                    .unwrap_or_else(|| input.q.clone());

                let page_ctx = crate::search::retrieve_context(
                    graph,
                    index,
                    &bfs_seed,
                    input.token_budget.unwrap_or(8192),
                    None,
                );
                context_text.push_str(
                    &page_ctx
                        .iter()
                        .map(|(_, _, text)| text.as_str())
                        .fold(
                            String::new(),
                            |mut acc, s| {
                                if !acc.is_empty() { acc.push('\n'); }
                                acc.push_str(s);
                                acc
                            },
                        ),
                );
                for (id, score, _) in &page_ctx {
                    let page_type = index
                        .get(id)
                        .map(|&idx| &graph[idx])
                        .map(|meta| meta.page_type.as_str())
                        .unwrap_or_default();
                    results.push(serde_json::json!({
                        "id": id, "score": score, "page_type": page_type, "type": "page",
                    }));
                }
            }

            if search_memory {
                let qp = crate::search::QueryParams {
                    query: input.q.clone(),
                    r#type: "memory".into(),
                    mode: "auto".into(),
                    limit: 10,
                    offset: 0,
                    recency: false,
                };
                if let Ok(resp) = crate::search::run_unified_search(&e, &qp) {
                    for r in &resp.results {
                        if let Ok(raw) = crate::page::get_page_raw(&e, &r.id) {
                            let (fm, body) = crate::parser::extract_frontmatter(&raw);
                            let title = fm.as_ref().and_then(|f| f.title.as_deref()).unwrap_or(&r.id);
                            let text = format!(
                                "[memory:{}] {} — {}\n",
                                r.id, title, body.trim()
                            );
                            let budget = input.token_budget.unwrap_or(8192);
                            if context_text.len() + text.len() <= budget {
                                context_text.push_str(&text);
                                results.push(serde_json::json!({
                                    "id": r.id,
                                    "score": r.score,
                                    "type": "memory",
                                }));
                            }
                        }
                    }
                }
            }

            Ok(serde_json::json!({
                "query": input.q,
                "token_budget": input.token_budget.unwrap_or(8192),
                "tokens_used": context_text.len() / 4,
                "result_count": results.len(),
                "results": results,
                "context": context_text,
            }))
        },
    );

    let e = engine.clone();
    registry.register_typed(
        "wm_search.resolve",
        "Resolve a query to a page ID",
        move |input: WmSearchResolveInput| {
            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let index = &snapshot.1;

            if let Some(&idx) = index.get(&input.q) {
                let meta = &graph[idx];
                return Ok(serde_json::json!({
                    "resolved": true,
                    "id": meta.id,
                    "title": meta.title,
                    "page_type": meta.page_type.as_str(),
                }));
            }

            for idx in graph.node_indices() {
                let meta = &graph[idx];
                if meta.title.eq_ignore_ascii_case(&input.q)
                    || meta.id.eq_ignore_ascii_case(&input.q)
                {
                    return Ok(serde_json::json!({
                        "resolved": true,
                        "id": meta.id,
                        "title": meta.title,
                        "page_type": meta.page_type.as_str(),
                    }));
                }
            }

            let bm25 = e.bm25_index.load();
            let bm25_results = bm25.search(&input.q, 5);
            if !bm25_results.is_empty() {
                let candidates: Vec<serde_json::Value> = bm25_results
                    .iter()
                    .map(|r| {
                        serde_json::json!({ "id": r.id, "score": r.score, "snippet": r.snippet })
                    })
                    .collect();
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
        },
    );
}
