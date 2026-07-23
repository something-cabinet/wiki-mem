use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

/// `POST /api/initial` – Returns basic engine state with graph stats, memory count, uptime, and staleness.
pub async fn get_initial(
    State(state): State<Arc<wm_core::engine::EngineState>>,
) -> Json<Value> {
    let snapshot = state.graph.load();
    let graph = &snapshot.0;
    let graph_node_count = graph.node_indices().count();
    let graph_edge_count = graph.edge_count();
    let session_memory_count = state.session_memory.len();
    let uptime_secs = state.started_at.elapsed().as_secs();
    let stale = state.stale_flag.load(Ordering::Relaxed);

    Json(json!({
        "success": true,
        "graph_node_count": graph_node_count,
        "graph_edge_count": graph_edge_count,
        "session_memory_count": session_memory_count,
        "uptime_secs": uptime_secs,
        "stale": stale,
    }))
}
