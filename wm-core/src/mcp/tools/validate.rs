use std::sync::Arc;

use crate::engine::EngineState;
use crate::mcp::transport::ToolRegistry;

/// Register validate tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
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
}
