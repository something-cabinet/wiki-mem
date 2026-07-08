use std::sync::Arc;

use serde_json::json;

use crate::engine::EngineState;
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;
use petgraph::visit::EdgeRef;

/// Register validate tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_with_schema(
        "wm_validate.check",
        "Validate wiki health — page completeness, broken wiki:* refs, orphan pages. Supports scope: all (default) or sdd.",
        json!({
            "type": "object",
            "properties": {
                "scope": { "type": "string", "description": "Validation scope: all (default) or sdd" }
            }
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let scope = args.optional_string("scope").unwrap_or_else(|| "all".into());

            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let index = &snapshot.1;
            let mut errors: Vec<serde_json::Value> = Vec::new();
            let mut warnings: Vec<serde_json::Value> = Vec::new();

            match scope.as_str() {
                "sdd" => {
                    // SDD validation: check spec pages have linked tasks
                    for idx in graph.node_indices() {
                        let meta = &graph[idx];
                        if meta.page_type != crate::engine::PageType::Spec {
                            continue;
                        }

                        // Check if any task page relates_to this spec
                        let has_linked_task = graph.node_indices().any(|other_idx| {
                            let other = &graph[other_idx];
                            if other.page_type != crate::engine::PageType::Task {
                                return false;
                            }
                            // Check if this task has a relates_to edge to the spec
                            // relates_to entries are formatted as "edge_type:wiki:subdir:slug"
                            other.relates_to.iter().any(|rel| {
                                rel.split_once(':')
                                    .map(|(_, target)| target.strip_prefix("wiki:").unwrap_or(target))
                                    .map(|t| t.replace(':', "/"))
                                    .map_or(false, |normalized| normalized == meta.id)
                                    || *rel == meta.id
                            })
                        });

                        if !has_linked_task {
                            warnings.push(serde_json::json!({
                                "id": meta.id,
                                "field": "sdd_coverage",
                                "message": format!("Spec '{}' has no linked tasks — add task pages with relates_to edges", meta.title)
                            }));
                        }

                        // Check if spec ACs have related task status
                        if !meta.acceptance_criteria.is_empty() {
                            // Simple check: warn if no task pages exist at all
                            let has_any_task = graph.node_indices().any(|i| {
                                graph[i].page_type == crate::engine::PageType::Task
                            });
                            if !has_any_task {
                                warnings.push(serde_json::json!({
                                    "id": meta.id,
                                    "field": "sdd_acceptance",
                                    "message": format!("Spec '{}' has acceptance criteria but no task pages found in wiki", meta.title)
                                }));
                            }
                        }
                    }

                    Ok(serde_json::json!({
                        "status": if errors.is_empty() { "pass" } else { "fail" },
                        "scope": "sdd",
                        "nodes": graph.node_count(),
                        "edges": graph.edge_count(),
                        "errors": errors,
                        "warnings": warnings,
                        "total_errors": errors.len(),
                    }))
                }
                _ => {
                    // Full wiki validation (default)
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
                        "scope": "all",
                        "nodes": graph.node_count(),
                        "edges": graph.edge_count(),
                        "errors": errors,
                        "warnings": warnings,
                        "total_errors": errors.len(),
                    }))
                }
            }
        }),
    );
}
