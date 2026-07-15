use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;

use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::transport::ToolRegistry;


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

    registry.register_typed(
        "wm_help",
        "Search tool documentation (optional: q=pattern)",
        move |input: WmHelpInput| {
            let q = input.q;

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
                ("wm_task", "Task operations: board, list, create, get, update, delete, check_ac, uncheck_ac, subtask"),
                ("wm_time", "Time tracking operations: start, stop, add, report"),
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
                ("wm_code.search", "Search code with tree-sitter AST queries"),
                ("wm_code.symbols", "Find symbols (functions, classes, types)"),
                ("wm_code.deps", "Find dependency relationships"),
                ("wm_ref.extract", "Extract @wiki/{type}/{name} references"),
                ("wm_ref.resolve", "Resolve a single @reference to its content"),
                ("wm_ref.resolve_all", "Resolve all @references in content"),
                ("wm_decision", "Manage architectural decision records (create, get)"),
                ("wm_skill.trigger", "Fire skills by lifecycle event"),
                ("skill.*", "Registered skill workflows"),
            ];

            let matched: Vec<serde_json::Value> = match q {
                Some(ref query) => {
                    let q_lower = query.to_lowercase();
                    let prefix = q_lower
                        .strip_suffix(".*")
                        .or_else(|| q_lower.strip_suffix('*'));
                    all_tools
                        .iter()
                        .filter(|(name, desc)| {
                            if let Some(p) = prefix {
                                return name.to_lowercase().contains(p);
                            }
                            name.to_lowercase().contains(&q_lower)
                                || desc.to_lowercase().contains(&q_lower)
                        })
                        .map(|(name, desc)| {
                            serde_json::json!({ "name": name, "description": desc })
                        })
                        .collect()
                }
                None => all_tools
                    .iter()
                    .map(|(name, desc)| {
                        serde_json::json!({ "name": name, "description": desc })
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
