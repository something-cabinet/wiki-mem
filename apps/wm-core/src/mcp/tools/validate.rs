use std::collections::HashMap;

use crate::mcp::prelude::*;

use crate::engine::{GraphEdge, WikiPageMeta};
use petgraph::visit::EdgeRef;

#[derive(Deserialize, JsonSchema)]
struct WmValidateCheckInput {
    #[schemars(description = "Validation scope: all (default) or sdd")]
    scope: Option<String>,
    #[schemars(description = "Validate a single page entity by ID (e.g., specs/auth)")]
    entity: Option<String>,
}

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_typed(
        "wm_validate.check",
        "Validate wiki health — page completeness, broken wiki:* refs, orphan pages. Supports scope: all (default) or sdd.",
        move |input: WmValidateCheckInput| {
            let scope = input.scope.as_deref().unwrap_or("all");
            let entity = input.entity.as_deref();

            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let index = &snapshot.1;
            let errors: Vec<serde_json::Value> = Vec::new();
            let warnings: Vec<serde_json::Value> = Vec::new();

            if let Some(entity_id) = entity {
                return validate_single_entity(graph, index, entity_id);
            }

            match scope {
                "sdd" => validate_sdd_scope(graph, errors, warnings),
                _ => validate_all_scope(graph, index, errors, warnings),
            }
        },
    );
}

fn validate_single_entity(
    graph: &petgraph::stable_graph::StableGraph<WikiPageMeta, GraphEdge>,
    index: &HashMap<String, petgraph::stable_graph::NodeIndex>,
    entity_id: &str,
) -> Result<serde_json::Value, ToolError> {
    let node_idx = match index.get(entity_id) {
        Some(idx) => *idx,
        None => {
            return Ok(serde_json::json!({
                "status": "fail",
                "entity": entity_id,
                "errors": [{"id": entity_id, "field": "id", "message": "Page not found"}],
                "warnings": [],
                "total_errors": 1,
            }))
        }
    };
    let meta = &graph[node_idx];
    let mut errors: Vec<serde_json::Value> = Vec::new();

    if meta.title.is_empty() {
        errors.push(
            serde_json::json!({"id": meta.id, "field": "title", "message": "Title is required"}),
        );
    }
    let stem = meta
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    if !stem.is_empty() {
        if let Ok(file_content) = std::fs::read_to_string(&meta.path) {
            let health = crate::parser::inspect_frontmatter_health(&file_content, &stem);
            if let Some(bad_id) = health.scientific_notation_id {
                errors.push(serde_json::json!({
                    "id": meta.id, "field": "id",
                    "message": format!(
                        "Frontmatter id '{}' looks like a scientific-notation number and will be corrupted on the next YAML round-trip — quote it: id: \"{}\"",
                        bad_id, bad_id
                    )
                }));
            }
            if health.duplicate_blocks {
                errors.push(serde_json::json!({
                    "id": meta.id, "field": "frontmatter",
                    "message": "File contains duplicate '---' frontmatter blocks — merge into a single block"
                }));
            }
            if meta.page_type == crate::engine::PageType::Task {
                if let Some(mismatch) = health.id_mismatch {
                    errors.push(serde_json::json!({
                        "id": meta.id, "field": "id",
                        "message": mismatch
                    }));
                }
            }
        }
    }
    for (_edge_type, target) in &meta.relates_to {
        let normalized = target
            .strip_prefix("wiki:")
            .unwrap_or(target)
            .replace(':', "/");
        if !index.contains_key(&normalized) && !index.contains_key(target) {
            errors.push(serde_json::json!({"id": meta.id, "field": "relates_to", "target": target, "message": format!("Broken wiki ref: '{}'", target)}));
        }
    }
    Ok(serde_json::json!({
        "status": if errors.is_empty() { "pass" } else { "fail" },
        "entity": entity_id,
        "errors": errors,
        "warnings": [],
        "total_errors": errors.len(),
    }))
}

fn validate_sdd_scope(
    graph: &petgraph::stable_graph::StableGraph<WikiPageMeta, GraphEdge>,
    errors: Vec<serde_json::Value>,
    mut warnings: Vec<serde_json::Value>,
) -> Result<serde_json::Value, ToolError> {
    for idx in graph.node_indices() {
        let meta = &graph[idx];
        if meta.page_type != crate::engine::PageType::Spec {
            continue;
        }

        let has_linked_task = graph.node_indices().any(|other_idx| {
            let other = &graph[other_idx];
            if other.page_type != crate::engine::PageType::Task {
                return false;
            }
            other.relates_to.iter().any(|(_edge_type, target)| {
                let normalized = target
                    .strip_prefix("wiki:")
                    .unwrap_or(target)
                    .replace(':', "/");
                normalized == meta.id || *target == meta.id
            })
        });

        if !has_linked_task {
            warnings.push(serde_json::json!({
                "id": meta.id,
                "field": "sdd_coverage",
                "message": format!("Spec '{}' has no linked tasks — add task pages with relates_to edges", meta.title)
            }));
        }

        if meta
            .task_data
            .as_ref()
            .map(|d| !d.acceptance_criteria.is_empty())
            .unwrap_or(false)
        {
            let has_any_task = graph
                .node_indices()
                .any(|i| graph[i].page_type == crate::engine::PageType::Task);
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

fn validate_all_scope(
    graph: &petgraph::stable_graph::StableGraph<WikiPageMeta, GraphEdge>,
    index: &HashMap<String, petgraph::stable_graph::NodeIndex>,
    mut errors: Vec<serde_json::Value>,
    mut warnings: Vec<serde_json::Value>,
) -> Result<serde_json::Value, ToolError> {
    for idx in graph.node_indices() {
        let meta = &graph[idx];

        if meta.title.is_empty() {
            errors.push(serde_json::json!({
                "id": meta.id, "field": "title", "message": "Title is required"
            }));
        }

        let stem = meta
            .path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if !stem.is_empty() {
            if let Ok(file_content) = std::fs::read_to_string(&meta.path) {
                let health = crate::parser::inspect_frontmatter_health(&file_content, &stem);
                if let Some(bad_id) = health.scientific_notation_id {
                    errors.push(serde_json::json!({
                        "id": meta.id, "field": "id",
                        "message": format!(
                            "Frontmatter id '{}' looks like a scientific-notation number and will be corrupted on the next YAML round-trip — quote it: id: \"{}\"",
                            bad_id, bad_id
                        )
                    }));
                }
                if health.duplicate_blocks {
                    errors.push(serde_json::json!({
                        "id": meta.id, "field": "frontmatter",
                        "message": "File contains duplicate '---' frontmatter blocks — merge into a single block"
                    }));
                }
                if meta.page_type == crate::engine::PageType::Task {
                    if let Some(mismatch) = health.id_mismatch {
                        warnings.push(serde_json::json!({
                            "id": meta.id, "field": "id",
                            "message": mismatch
                        }));
                    }
                }
            }
        }

        match meta.page_type {
            crate::engine::PageType::Task => {
                if meta
                    .task_data
                    .as_ref()
                    .map(|d| d.acceptance_criteria.is_empty())
                    .unwrap_or(true)
                {
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
            crate::engine::PageType::Spec
                if meta.status == crate::engine::PageStatus::Draft
                    && meta
                        .spec_data
                        .as_ref()
                        .map(|d| d.stakeholders.is_empty())
                        .unwrap_or(true) =>
            {
                warnings.push(serde_json::json!({
                    "id": meta.id, "field": "stakeholders",
                    "message": "Spec should have stakeholders defined"
                }));
            }
            crate::engine::PageType::Decision if meta.decision_data.is_none() => {
                warnings.push(serde_json::json!({
                    "id": meta.id, "field": "decision",
                    "message": "Decision page should have context, options, rationale"
                }));
            }
            crate::engine::PageType::Pattern if meta.pattern_data.is_none() => {
                warnings.push(serde_json::json!({
                    "id": meta.id, "field": "pattern",
                    "message": "Pattern page should have when_to_use and example"
                }));
            }
            crate::engine::PageType::Rule if meta.rule_data.is_none() => {
                warnings.push(serde_json::json!({
                    "id": meta.id, "field": "rule",
                    "message": "Rule page should have category and rationale"
                }));
            }
            _ => {}
        }

        for (_edge_type, target) in &meta.relates_to {
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
