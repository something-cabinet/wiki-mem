use crate::mcp::prelude::*;


// ─── Input structs ─────────────────────────────────────────────────

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

// ─── Registration ──────────────────────────────────────────────────

/// Register project/initial/help tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    // ─── wm_initial ────────────────────────────────────────────

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
                *page_types.entry(type_name.to_string()).or_insert(0) += 1;
            }

            let mut source_states: std::collections::BTreeMap<String, usize> =
                std::collections::BTreeMap::new();
            let registry_lock = e
                .source_registry
                .read()
                .map_err(|_| ToolError::lock_poisoned("registry"))?;
            for entry in registry_lock.values() {
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
                "page_types_available": ["task", "spec", "concept", "pattern", "decision", "howto", "reference", "note"],
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
        },
    );

    // ─── wm_help ───────────────────────────────────────────────

    let e = engine.clone();
    registry.register_typed(
        "wm_help",
        "Search tool documentation (optional: q=pattern)",
        move |input: WmHelpInput| {
            let q = input.q;
            let tools = e.tool_list.read().map_err(|_| ToolError::lock_poisoned("tool_list"))?;

            let matched: Vec<serde_json::Value> = match q {
                Some(ref query) => {
                    let q_lower = query.to_lowercase();
                    let prefix = q_lower
                        .strip_suffix(".*")
                        .or_else(|| q_lower.strip_suffix('*'));
                    tools
                        .iter()
                        .filter(|tool| {
                            let name = tool.name.to_lowercase();
                            let desc = tool.description.as_deref().unwrap_or_default().to_lowercase();
                            if let Some(p) = prefix {
                                return name.contains(p);
                            }
                            name.contains(&q_lower) || desc.contains(&q_lower)
                        })
                        .map(|tool| {
                            serde_json::json!({
                                "name": tool.name,
                                "description": tool.description,
                                "schema": tool.input_schema,
                            })
                        })
                        .collect()
                }
                None => tools
                    .iter()
                    .map(|tool| {
                        serde_json::json!({
                            "name": tool.name,
                            "description": tool.description,
                            "schema": tool.input_schema,
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

    // ─── Project Tools ─────────────────────────────────────────

    let e = engine.clone();
    registry.register_typed(
        "wm_project.status",
        "Project status information",
        move |_input: EmptyInput| {
            let root = std::env::current_dir().ok();
            let mut resp = serde_json::json!({
                "project": root.as_ref().map(|r| r.join(".wm").join("config.json").exists()).unwrap_or(false).then(|| "active").unwrap_or("none"),
                "root": root.map(|r| r.to_string_lossy().to_string()),
            });

            // Include LSP and git_tracking config when available
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
            if !root.join(".wm").join("wm_config.json").exists() {
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
