use axum::{
    extract::{Path, State},
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

pub async fn graph_neighbors(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<Value> {
    let snapshot = state.engine.graph.load();
    let graph = &snapshot.0;
    let index = &snapshot.1;
    if let Some(&node_idx) = index.get(&id) {
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
            "center_id": id,
            "neighbors": neighbors,
        }))
    } else {
        Json(json!({
            "success": false,
            "error": "Node not found",
        }))
    }
}
