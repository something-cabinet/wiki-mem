use serde_json::Value;
use petgraph::visit::EdgeRef;
use wm_core::engine::EngineState;
use tauri::State;

#[tauri::command]
fn get_initial(state: State<'_, EngineState>) -> Result<Value, String> {
    let snapshot = state.graph.load();
    let graph = &snapshot.0;
    let elapsed = state.started_at.elapsed().as_secs();

    Ok(serde_json::json!({
        "success": true,
        "project": "WM Wiki Memory Engine",
        "graph_node_count": graph.node_count(),
        "graph_edge_count": graph.edge_count(),
        "session_memory_count": state.session_memory.len(),
        "uptime_secs": elapsed,
        "stale": state.stale_flag.load(std::sync::atomic::Ordering::Acquire),
    }))
}

#[tauri::command]
fn get_graph_full(state: State<'_, EngineState>) -> Result<Value, String> {
    let snapshot = state.graph.load();
    let graph = &snapshot.0;

    let nodes: Vec<Value> = graph
        .node_indices()
        .map(|idx| {
            let meta = &graph[idx];
            let degree = graph.edges(idx).count()
                + graph.edges_directed(idx, petgraph::Direction::Incoming).count();
            serde_json::json!({
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
            Some(serde_json::json!({
                "source": graph[source].id,
                "target": graph[target].id,
                "edge_type": format!("{:?}", graph[edge_idx]).to_lowercase(),
            }))
        })
        .collect();

    Ok(serde_json::json!({
        "success": true,
        "nodes": nodes,
        "edges": edges,
        "node_count": nodes.len(),
        "edge_count": edges.len(),
    }))
}

#[tauri::command]
fn get_graph_stats(state: State<'_, EngineState>) -> Result<Value, String> {
    let snapshot = state.graph.load();
    let graph = &snapshot.0;
    Ok(serde_json::json!({
        "success": true,
        "node_count": graph.node_count(),
        "edge_count": graph.edge_count(),
    }))
}

#[tauri::command]
fn get_graph_neighbors(state: State<'_, EngineState>, id: String) -> Result<Value, String> {
    let snapshot = state.graph.load();
    let graph = &snapshot.0;
    let index = &snapshot.1;

    let node_idx = index.get(&id)
        .ok_or_else(|| format!("Node not found: {}", id))?;

    let mut neighbors: Vec<Value> = Vec::new();
    for edge in graph.edges(*node_idx) {
        let target = edge.target();
        let meta = &graph[target];
        neighbors.push(serde_json::json!({
            "id": meta.id,
            "title": meta.title,
            "page_type": meta.page_type,
            "edge_type": format!("{:?}", edge.weight()).to_lowercase(),
        }));
    }

    Ok(serde_json::json!({
        "success": true,
        "center_id": id,
        "neighbors": neighbors,
    }))
}
