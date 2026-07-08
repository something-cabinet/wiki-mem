use std::sync::Arc;

use crate::engine::EngineState;
use crate::mcp::transport::ToolRegistry;
use petgraph::visit::EdgeRef;

/// Register validate tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_with_desc(
        "validate.check",
        "Validate wiki health — page completeness, broken wiki:* refs, orphan pages",
        Arc::new(move |_params| {
            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let index = &snapshot.1;
            let mut errors: Vec<serde_json::Value> = Vec::new();
            let mut warnings: Vec<serde_json::Value> = Vec::new();

            for idx in graph.node_indices() {
                let meta = &graph[idx];

                // Page completeness checks
                if meta.title.is_empty() {
                    errors.push(serde_json::json!({
                        "id": meta.id, "field": "title", "message": "Title is required"
                    }));
                }

                match meta.page_type {
                    crate::engine::PageType::Task => {
                        if meta.acceptance_criteria.is_empty() {
                            errors.push(serde_json::json!({
                                "id": meta.id, "field": "acceptance_criteria",
                                "message": "Task should have at least one acceptance criterion"
                            }));
                        }
                        if meta.assignee.is_none() {
                            warnings.push(serde_json::json!({
                                "id": meta.id, "field": "assignee",
                                "message": "Task should have an assignee"
                            }));
                        }
                    }
                    crate::engine::PageType::Spec => {
                        if meta.status == crate::engine::PageStatus::Draft
                            && meta.stakeholders.is_empty()
                        {
                            warnings.push(serde_json::json!({
                                "id": meta.id, "field": "stakeholders",
                                "message": "Spec should have stakeholders defined"
                            }));
                        }
                    }
                    crate::engine::PageType::Decision => {
                        if meta.decision.is_none() {
                            warnings.push(serde_json::json!({
                                "id": meta.id, "field": "decision",
                                "message": "Decision page should have context, options, rationale"
                            }));
                        }
                    }
                    crate::engine::PageType::Pattern => {
                        if meta.pattern.is_none() {
                            warnings.push(serde_json::json!({
                                "id": meta.id, "field": "pattern",
                                "message": "Pattern page should have when_to_use and example"
                            }));
                        }
                    }
                    _ => {}
                }

                // Check relates_to targets resolve to existing graph nodes
                // Format: "edge_type:wiki:subdir:slug" e.g. "example_of:wiki:concepts:graph-architecture"
                for rel in &meta.relates_to {
                    // Split on first colon to get edge type and target
                    if let Some(target) = rel.split_once(':').map(|(_, t)| t) {
                        // Target may be "wiki:concepts:graph-architecture" or "concepts/graph-architecture"
                        // Normalize: "wiki:concepts:graph-architecture" → "concepts/graph-architecture"
                        let normalized = if let Some(rest) = target.strip_prefix("wiki:") {
                            rest.replace(':', "/")
                        } else {
                            target.to_string()
                        };
                        if !index.contains_key(&normalized) && !index.contains_key(target) {
                            errors.push(serde_json::json!({
                                "id": meta.id,
                                "field": "relates_to",
                                "target": target,
                                "normalized": normalized,
                                "message": format!("Broken wiki ref: '{}' — page '{}' not found", target, normalized)
                            }));
                        }
                    }
                }
            }

            // Orphan check: pages with no incoming edges
            let mut has_incoming: std::collections::HashSet<&str> = std::collections::HashSet::new();
            for idx in graph.node_indices() {
                for edge in graph.edges(idx) {
                    has_incoming.insert(graph[edge.target()].id.as_str());
                }
            }
            for meta in graph.node_indices().map(|idx| &graph[idx]) {
                if meta.page_type != crate::engine::PageType::Task
                    && !has_incoming.contains(meta.id.as_str())
                {
                    warnings.push(serde_json::json!({
                        "id": meta.id,
                        "field": "orphan",
                        "message": "Page has no incoming links from other pages"
                    }));
                }
            }

            Ok(serde_json::json!({
                "status": if errors.is_empty() { "pass" } else { "fail" },
                "nodes": graph.node_count(),
                "edges": graph.edge_count(),
                "errors": errors,
                "warnings": warnings,
                "total_errors": errors.len(),
            }))
        }),
    );
}
