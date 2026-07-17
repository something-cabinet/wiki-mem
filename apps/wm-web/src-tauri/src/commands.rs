use petgraph::visit::EdgeRef;
use serde::Deserialize;
use serde_json::Value;
use tauri::State;
use wm_core::engine::EngineState;
use wm_core::search::{self, QueryParams};

// ─── Initial ─────────────────────────────────────────

#[tauri::command]
pub fn get_initial(state: State<'_, EngineState>) -> Result<Value, String> {
    let snapshot = state.graph.load();
    let graph = &snapshot.0;
    let elapsed = state.started_at.elapsed().as_secs();
    Ok(serde_json::json!({
        "success": true, "project": "WM Wiki Memory Engine",
        "graph_node_count": graph.node_count(), "graph_edge_count": graph.edge_count(),
        "session_memory_count": state.session_memory.len(),
        "uptime_secs": elapsed,
        "stale": state.stale_flag.load(std::sync::atomic::Ordering::Acquire),
    }))
}

// ─── Search ──────────────────────────────────────────

#[derive(Deserialize)]
pub struct SearchPayload {
    pub q: String,
    #[serde(rename = "type")]
    pub r#type: Option<String>,
    pub mode: Option<String>,
    pub limit: Option<usize>,
}

#[tauri::command]
pub fn search(state: State<'_, EngineState>, payload: SearchPayload) -> Result<Value, String> {
    let qp = QueryParams {
        query: payload.q,
        r#type: payload.r#type.unwrap_or_else(|| "all".into()),
        mode: payload.mode.unwrap_or_else(|| "auto".into()),
        limit: payload.limit.unwrap_or(20),
        offset: 0,
        recency: true,
    };
    match search::query::run_unified_search(&state, &qp) {
        Ok(results) => {
            let items: Vec<Value> = results.iter().map(|r| serde_json::json!({
                "id": r.id, "score": r.score, "type": r.r#type,
                "page_type": r.page_type, "page_type_rank": r.page_type_rank,
                "centrality": r.centrality, "snippet": r.snippet,
            })).collect();
            Ok(serde_json::json!({ "success": true, "results": items, "total": items.len() }))
        }
        Err(e) => Ok(serde_json::json!({ "success": false, "error": e })),
    }
}

// ─── Graph ───────────────────────────────────────────

#[tauri::command]
pub fn get_graph_full(state: State<'_, EngineState>) -> Result<Value, String> {
    let snapshot = state.graph.load();
    let graph = &snapshot.0;
    let nodes: Vec<Value> = graph.node_indices().map(|idx| {
        let meta = &graph[idx];
        let degree = graph.edges(idx).count() + graph.edges_directed(idx, petgraph::Direction::Incoming).count();
        serde_json::json!({ "id": meta.id, "title": meta.title, "page_type": meta.page_type, "degree": degree })
    }).collect();
    let edges: Vec<Value> = graph.edge_indices().filter_map(|ei| {
        let (s, t) = graph.edge_endpoints(ei)?;
        Some(serde_json::json!({ "source": graph[s].id, "target": graph[t].id, "edge_type": format!("{:?}", graph[ei]).to_lowercase() }))
    }).collect();
    Ok(serde_json::json!({ "success": true, "nodes": nodes, "edges": edges, "node_count": nodes.len(), "edge_count": edges.len() }))
}

#[tauri::command]
pub fn get_graph_stats(state: State<'_, EngineState>) -> Result<Value, String> {
    let snapshot = state.graph.load();
    let graph = &snapshot.0;
    Ok(serde_json::json!({ "success": true, "node_count": graph.node_count(), "edge_count": graph.edge_count() }))
}

#[derive(Deserialize)]
pub struct NeighborPayload {
    pub id: String,
}

#[tauri::command]
pub fn get_graph_neighbors(state: State<'_, EngineState>, payload: NeighborPayload) -> Result<Value, String> {
    let snapshot = state.graph.load();
    let graph = &snapshot.0;
    let index = &snapshot.1;
    let node_idx = index.get(&payload.id).ok_or_else(|| format!("Node not found: {}", payload.id))?;
    let mut neighbors: Vec<Value> = Vec::new();
    for edge in graph.edges(*node_idx) {
        let target = edge.target();
        let meta = &graph[target];
        neighbors.push(serde_json::json!({ "id": meta.id, "title": meta.title, "page_type": meta.page_type, "edge_type": format!("{:?}", edge.weight()).to_lowercase() }));
    }
    Ok(serde_json::json!({ "success": true, "center_id": payload.id, "neighbors": neighbors }))
}
