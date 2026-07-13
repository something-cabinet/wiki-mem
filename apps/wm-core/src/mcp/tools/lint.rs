use petgraph::visit::EdgeRef;
use schemars::JsonSchema;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::engine::EngineState;
use crate::mcp::transport::ToolRegistry;
use crate::mcp::typed::TypedRegister;

// ─── Input types ───────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
struct WmLintCheckInput {}

#[derive(Deserialize, JsonSchema)]
struct WmLintFixInput {}

/// Register lint tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_read(
        "wm_lint.check",
        "Check wiki for common issues",
        move |_input: WmLintCheckInput| {
            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let index = &snapshot.1;
            let mut issues = Vec::new();
            let mut cycle_info = None;

            for idx in graph.node_indices() {
                let meta = &graph[idx];

                let has_inbound = graph.edges_directed(idx, petgraph::Direction::Incoming).count() > 0;
                if !has_inbound {
                    issues.push(serde_json::json!({
                        "type": "orphan",
                        "severity": "warning",
                        "id": meta.id,
                        "message": "No inbound edges — consider adding relationships"
                    }));
                }

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

                if meta.page_type == crate::engine::PageType::Task && meta.acceptance_criteria.is_empty() {
                    issues.push(serde_json::json!({
                        "type": "missing_acs",
                        "severity": "warning",
                        "id": meta.id,
                        "message": "Task has no acceptance criteria"
                    }));
                }

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

            let registry_lock = e.source_registry.read().map_err(|_| crate::error::ToolError::lock_poisoned("registry"))?;
            let mut stale_count = 0usize;
            for entry in registry_lock.values() {
                let is_stale = if entry.state == crate::engine::SourceState::Stale {
                    true
                } else if entry.stored_path.exists() {
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
        },
    );

    let e = engine.clone();
    registry.register_write(
        "wm_lint.fix",
        "Auto-fix common issues",
        move |_input: WmLintFixInput| {
            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let fixed = crate::graph::auto_fix_missing_frontmatter(graph, &e.write_channel);

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
                    e2.rebuild_memory_index_from_disk(&memory_dir);
                });
            }

            Ok(serde_json::json!({
                "fixed": fixed,
                "message": format!("Fixed {} issue(s)", fixed),
            }))
        },
    );
}
