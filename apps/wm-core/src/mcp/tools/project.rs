use crate::mcp::prelude::*;
use wm_constants::*;

/// Session runtime context injected into the `wm_initial` response.
///
/// Previously the MCP *transport* appended this as an extra text block on the
/// first tool call. With the stdio→HTTP proxy (task #41) the transport is a
/// dumb forwarder, so the injection lives in the `wm_initial` handler itself:
/// the daemon emits it as a `runtime_context` field that the proxy passes
/// through verbatim.
fn runtime_context(engine: &EngineState) -> String {
    let version = env!("CARGO_PKG_VERSION");
    let snapshot = engine.graph.load();
    let graph = &snapshot.0;

    let core_titles: Vec<&str> = graph
        .node_indices()
        .filter(|i| graph[*i].page_type == crate::engine::PageType::Core)
        .map(|i| graph[i].title.as_str())
        .take(5)
        .collect();
    let core_count = graph
        .node_indices()
        .filter(|i| graph[*i].page_type == crate::engine::PageType::Core)
        .count();
    let active_task_count = graph
        .node_indices()
        .filter(|i| {
            use crate::engine::{PageStatus, PageType};
            graph[*i].page_type == PageType::Task
                && (graph[*i].status == PageStatus::Todo
                    || graph[*i].status == PageStatus::InProgress
                    || graph[*i].status == PageStatus::Blocked)
        })
        .count();

    let core_line = if core_titles.is_empty() {
        format!("Core pages: {}", core_count)
    } else {
        format!("Core pages: {} ({})", core_titles.join(", "), core_count)
    };

    format!(
        "[Wiki Memory Engine v{}]\n{} | Active tasks: {}\n",
        version, core_line, active_task_count
    )
}

#[derive(Deserialize, JsonSchema)]
struct EmptyInput {}

#[derive(Deserialize, JsonSchema)]
struct WmHelpInput {
    #[schemars(description = "Optional search pattern to filter tools")]
    q: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct WmProjectSetInput {
    #[schemars(description = "Path to project root directory")]
    path: String,
}

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_typed(
        "wm_initial",
        "Get project state, graph stats, and model status",
        move |_input: EmptyInput| {
            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let node_count = graph.node_count();
            let model_loaded = e.embedder.is_loaded();

            let mut page_types: std::collections::BTreeMap<String, usize> =
                std::collections::BTreeMap::new();
            for idx in graph.node_indices() {
                let type_name = graph[idx].page_type.as_str();
                let counter = page_types.entry(type_name.to_string()).or_insert(0);
                *counter = counter.wrapping_add(1);
            }

            let mut source_states: std::collections::BTreeMap<String, usize> =
                std::collections::BTreeMap::new();
            let registry_lock = e
                .source_registry
                .read()
                .map_err(|_| ToolError::lock_poisoned("registry"))?;
            for entry in registry_lock.values() {
                let state_name = format!("{:?}", entry.state).to_lowercase();
                let counter = source_states.entry(state_name).or_insert(0);
                *counter = counter.wrapping_add(1);
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
                "page_types_available": crate::engine::PageType::all_type_names(),
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
                },
                "runtime_context": runtime_context(&e),
            }))
        },
    );

    let e = engine.clone();
    registry.register_typed(
        "wm_help",
        "Search tool documentation (optional: q=pattern)",
        move |input: WmHelpInput| {
            let q = input.q;
            let tools = e.tool_list.read().map_err(|_| ToolError::lock_poisoned("tool_list"))?;

            fn tool_name(tool: &serde_json::Value) -> &str {
                tool.get("name")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("")
            }
            fn tool_desc(tool: &serde_json::Value) -> &str {
                tool.get("description")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("")
            }

            let matched: Vec<serde_json::Value> = match q {
                Some(ref query) => {
                    let q_lower = query.to_lowercase();
                    let prefix = q_lower
                        .strip_suffix(".*")
                        .or_else(|| q_lower.strip_suffix('*'));
                    tools
                        .iter()
                        .filter(|tool| {
                            let name = tool_name(tool).to_lowercase();
                            let desc = tool_desc(tool).to_lowercase();
                            if let Some(p) = prefix {
                                return name.contains(p);
                            }
                            name.contains(&q_lower) || desc.contains(&q_lower)
                        })
                        .map(|tool| {
                            serde_json::json!({
                                "name": tool_name(tool),
                                "description": tool_desc(tool),
                                "schema": tool.get("inputSchema"),
                            })
                        })
                        .collect()
                }
                None => tools
                    .iter()
                    .map(|tool| {
                        serde_json::json!({
                            "name": tool_name(tool),
                            "description": tool_desc(tool),
                            "schema": tool.get("inputSchema"),
                        })
                    })
                    .collect(),
            };

            Ok(serde_json::json!({
                "available_tools": matched,
                "total": matched.len(),
                "documentation": "Use wm_help with q=<search> to filter tools by name or description."
            }))
        },
    );

    let e = engine.clone();
    registry.register_typed(
        "wm_project.status",
        "Project status information",
        move |_input: EmptyInput| {
            let root = std::env::current_dir().ok();
            let mut resp = serde_json::json!({
                "project": if root.as_ref().map(|r| r.join(WM_DIR).join(CONFIG_FILE).exists()).unwrap_or(false) { "active" } else { "none" },
                "root": root.map(|r| r.to_string_lossy().to_string()),
            });

            if let Ok(cfg) = e.config.read() {
                if let Some(ref lsp) = cfg.lsp {
                    resp["lsp"] = serde_json::json!(lsp);
                }
                if let Some(ref git) = cfg.git_tracking {
                    resp["gitTracking"] = serde_json::json!(git);
                }
            }

            Ok(resp)
        },
    );

    registry.register_typed(
        "wm_project.detect",
        "Detect project root from current directory",
        move |_input: EmptyInput| {
            let root = crate::config::detect_project_root();
            match root {
                Some(path) => Ok(serde_json::json!({
                    "project": "detected",
                    "root": path.to_string_lossy().to_string(),
                })),
                None => Err(crate::error::ToolError::not_found(
                    "project",
                    "No .wm/config.json found in current or parent directories. Run 'wm init' first.",
                )),
            }
        },
    );

    let e = engine.clone();
    registry.register_typed(
        "wm_project.set",
        "Set the current project root",
        move |input: WmProjectSetInput| {
            let path = input.path;
            let root = std::path::PathBuf::from(&path);
            if !root.join(WM_DIR).join("wm_config.json").exists() {
                return Err(crate::error::ToolError::not_found(
                    "project",
                    &format!("No .wm/config.json found at {}", root.display()),
                ));
            }
            e.project_root
                .write()
                .map_err(|_| ToolError::lock_poisoned("project_root"))?
                .clone_from(&root);
            Ok(serde_json::json!({ "project": "set", "root": root.to_string_lossy().to_string() }))
        },
    );
}
