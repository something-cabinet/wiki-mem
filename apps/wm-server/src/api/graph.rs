use axum::{
    extract::State,
    Json,
};
use petgraph::visit::EdgeRef;
use serde_json::{json, Value};
use wm_core::engine::WikiPageMeta;
use crate::AppState;

pub async fn graph_stats(
    State(state): State<AppState>,
) -> Json<Value> {
    let snapshot = state.engine.graph.load();
    let graph = &snapshot.0;
    Json(json!({
        "success": true,
        "node_count": graph.node_count(),
        "edge_count": graph.edge_count(),
    }))
}

#[derive(serde::Deserialize)]
pub struct NeighborPayload {
    id: String,
}

pub async fn graph_full(
    State(state): State<AppState>,
) -> Json<Value> {
    let snapshot = state.engine.graph.load();
    let graph = &snapshot.0;
    let index = &snapshot.1;

    let nodes: Vec<Value> = graph
        .node_indices()
        .map(|idx| {
            let meta = &graph[idx];
            let degree = graph.edges(idx).count()
                + graph.edges_directed(idx, petgraph::Direction::Incoming).count();
            json!({
                "id": meta.id,
                "title": meta.title,
                "page_type": meta.page_type,
                "degree": degree,
            })
        })
        .collect();

    let edges: Vec<Value> = graph
        .edge_indices()
        .filter_map(|edge_idx| {
            let (source, target) = graph.edge_endpoints(edge_idx)?;
            let edge_type = &graph[edge_idx];
            Some(json!({
                "source": graph[source].id,
                "target": graph[target].id,
                "edge_type": format!("{:?}", edge_type).to_lowercase(),
            }))
        })
        .collect();

    Json(json!({
        "success": true,
        "nodes": nodes,
        "edges": edges,
        "node_count": nodes.len(),
        "edge_count": edges.len(),
    }))
}

pub async fn graph_neighbors(
    State(state): State<AppState>,
    Json(payload): Json<NeighborPayload>,
) -> Json<Value> {
    let snapshot = state.engine.graph.load();
    let graph = &snapshot.0;
    let index = &snapshot.1;
    if let Some(&node_idx) = index.get(&payload.id) {
        let mut neighbors: Vec<Value> = Vec::new();
        for edge in graph.edges(node_idx) {
            let target = edge.target();
            let meta: &WikiPageMeta = &graph[target];
            neighbors.push(json!({
                "id": meta.id,
                "title": meta.title,
                "page_type": meta.page_type,
                "edge_type": edge.weight(),
            }));
        }
        Json(json!({
            "success": true,
            "center_id": payload.id.clone(),
            "neighbors": neighbors,
        }))
    } else {
        Json(json!({
            "success": false,
            "error": format!("Node not found: {}", payload.id),
        }))
    }
}
