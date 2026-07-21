use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

/// `POST /api/graph/stats` – Returns graph-wide statistics.
pub async fn stats(
    State(state): State<Arc<wm_core::engine::EngineState>>,
) -> Json<Value> {
    let snapshot = state.graph.load();
    let graph = &snapshot.0;
    let mut node_count = 0i64;
    let mut type_counts: HashMap<String, i64> = HashMap::new();

    for node in graph.node_indices() {
        node_count += 1;
        let meta = &graph[node];
        *type_counts.entry(meta.page_type.as_str().to_string()).or_default() += 1;
    }
    let edge_count = graph.edge_count() as i64;

    Json(json!({
        "success": true,
        "node_count": node_count,
        "edge_count": edge_count,
        "type_counts": type_counts,
    }))
}

/// `POST /api/graph/full` – Returns the full graph structure.
pub async fn full(
    State(state): State<Arc<wm_core::engine::EngineState>>,
) -> Json<Value> {
    let snapshot = state.graph.load();
    let graph = &snapshot.0;
    let id_index = &snapshot.1;

    // Build node data with degree and reverse index for O(1) lookups
    let nodes: Vec<Value> = graph
        .node_indices()
        .map(|i| {
            let meta = &graph[i];
            json!({
                "id": meta.id,
                "title": meta.title,
                "page_type": meta.page_type,
                "degree": graph.neighbors(i).count(),
            })
        })
        .collect();

    let edges: Vec<Value> = graph
        .edge_indices()
        .filter_map(|i| {
            let (source, target) = graph.edge_endpoints(i)?;
            let edge = &graph[i];
            let source_id = id_index.iter().find(|(_, &idx)| idx == source).map(|(id, _)| id.clone());
            let target_id = id_index.iter().find(|(_, &idx)| idx == target).map(|(id, _)| id.clone());
            // Serialize edge_type as kebab-case to match CSS --edge-type-* tokens
            let edge_type_str = serde_json::to_value(edge)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| format!("{:?}", edge));
            Some(json!({
                "source": source_id.unwrap_or_default(),
                "target": target_id.unwrap_or_default(),
                "edge_type": edge_type_str,
            }))
        })
        .collect();

    Json(json!({
        "success": true,
        "node_count": nodes.len(),
        "edge_count": edges.len(),
        "nodes": nodes,
        "edges": edges,
    }))
}

#[derive(Deserialize)]
pub struct NeighborsInput {
    pub id: String,
    #[allow(dead_code)]
    pub depth: Option<usize>,
}

pub async fn neighbors(
    State(state): State<Arc<wm_core::engine::EngineState>>,
    Json(input): Json<NeighborsInput>,
) -> Json<Value> {
    let snapshot = state.graph.load();
    let graph = &snapshot.0;
    let id_index = &snapshot.1;

    match id_index.get(&input.id) {
        Some(&node_idx) => {
            let items: Vec<Value> = graph
                .neighbors(node_idx)
                .map(|n| {
                    let meta = &graph[n];
                    json!({"id": meta.id, "title": meta.title, "page_type": meta.page_type})
                })
                .collect();
            Json(json!({"success": true, "neighbors": items}))
        }
        None => Json(json!({"success": true, "neighbors": []})),
    }
}

#[derive(Deserialize)]
pub struct PathInput {
    pub start: String,
    pub end: String,
    pub max_depth: Option<usize>,
}

pub async fn path(
    State(state): State<Arc<wm_core::engine::EngineState>>,
    Json(input): Json<PathInput>,
) -> Json<Value> {
    let snapshot = state.graph.load();
    let graph = &snapshot.0;
    let id_index = &snapshot.1;
    let max_depth = input.max_depth.unwrap_or(5);

    match (id_index.get(&input.start), id_index.get(&input.end)) {
        (Some(&start_idx), Some(&end_idx)) => {
            let result =
                wm_core::graph::find_path(graph, id_index, start_idx, end_idx, max_depth);
            let path: Vec<Value> = result
                .into_iter()
                .map(|(id, title, _edge_type)| {
                    json!({"id": id, "title": title})
                })
                .collect();
            Json(json!({"success": true, "path": path}))
        }
        _ => Json(json!({"success": true, "path": []})),
    }
}

#[derive(Deserialize)]
pub struct SubgraphInput {
    pub center: String,
    pub depth: Option<usize>,
}

pub async fn subgraph(
    State(state): State<Arc<wm_core::engine::EngineState>>,
    Json(input): Json<SubgraphInput>,
) -> Json<Value> {
    let depth = input.depth.unwrap_or(2);
    let snapshot = state.graph.load();
    let graph = &snapshot.0;
    let id_index = &snapshot.1;

    match id_index.get(&input.center) {
        Some(&center_idx) => {
            use petgraph::visit::Bfs;
            let mut bfs = Bfs::new(graph, center_idx);
            let mut node_ids = std::collections::HashSet::new();
            let mut current_depth = 0;

            while let Some(nx) = bfs.next(graph) {
                node_ids.insert(nx);
                if node_ids.len() > 100 || current_depth > depth {
                    break;
                }
                current_depth += 1;
            }

            let nodes: Vec<Value> = node_ids
                .iter()
                .map(|&i| {
                    let meta = &graph[i];
                    json!({"id": meta.id, "title": meta.title, "page_type": meta.page_type})
                })
                .collect();

            let edges: Vec<Value> = graph
                .edge_indices()
                .filter_map(|e| {
                    let (source, target) = graph.edge_endpoints(e)?;
                    if node_ids.contains(&source) && node_ids.contains(&target) {
                        let edge = &graph[e];
                        Some(json!({
                            "source": graph[source].id.clone(),
                            "target": graph[target].id.clone(),
                            "edge_type": format!("{:?}", edge),
                        }))
                    } else {
                        None
                    }
                })
                .collect();

            Json(json!({"success": true, "nodes": nodes, "edges": edges}))
        }
        None => Json(json!({"success": true, "nodes": [], "edges": []})),
    }
}
