use axum::{extract::State, Json};
use serde_json::{json, Value};
use std::sync::Arc;


pub async fn status(State(state): State<Arc<wm_core::engine::EngineState>>) -> Json<Value> {
    let snapshot = state.graph.load();
    let graph = &snapshot.0;
    Json(json!({
        "success": true,
        "graph_nodes": graph.node_count(),
        "graph_edges": graph.edge_count(),
        "bm25_indexed": true,
        "vectors_persisted": 0,
    }))
}
