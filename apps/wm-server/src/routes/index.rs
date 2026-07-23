use std::sync::Arc;
use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct RebuildInput {
    #[allow(dead_code)]
    pub skip_embed: Option<bool>,
}

pub async fn rebuild(
    State(state): State<Arc<wm_core::engine::EngineState>>,
    Json(_input): Json<RebuildInput>,
) -> Json<Value> {
    let wiki_dir = state.project_root.read().unwrap().join(".wm").join("wiki");
    let node_count = state.rebuild_graph(&wiki_dir);
    Json(json!({"success": true, "nodes": node_count}))
}

pub async fn status(
    State(state): State<Arc<wm_core::engine::EngineState>>,
) -> Json<Value> {
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
