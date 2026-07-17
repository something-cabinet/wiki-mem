use petgraph::visit::EdgeRef;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use tauri::State;
use wm_core::engine::{EngineState, WikiPageMeta};
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

// ─── Helper ─────────────────────────────────────────

fn wiki_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_default().join(".wm").join("wiki")
}

fn read_all_pages(dir: &PathBuf) -> Vec<WikiPageMeta> {
    if !dir.exists() { return vec![]; }
    let mut pages = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                let content = std::fs::read_to_string(&path).unwrap_or_default();
                if let Some((fm, _)) = content.split_once("---").and_then(|s| {
                    s.1.find("---").map(|end| (&s.1[..end], &s.1[end+3..]))
                }) {
                    if let Ok(meta) = serde_yaml::from_str::<WikiPageMeta>(fm) {
                        pages.push(meta);
                    }
                }
            }
        }
    }
    pages
}

// ─── Pages ──────────────────────────────────────────

#[tauri::command]
pub fn list_pages() -> Result<Value, String> {
    let dir = wiki_dir();
    let pages = read_all_pages(&dir);
    let items: Vec<Value> = pages.iter().map(|p| serde_json::json!({
        "id": p.id, "title": p.title, "type": p.page_type, "status": p.status,
    })).collect();
    Ok(serde_json::json!({ "success": true, "pages": items }))
}

#[derive(Deserialize)]
pub struct GetPagePayload {
    pub id: String,
}

#[tauri::command]
pub fn get_page(payload: GetPagePayload) -> Result<Value, String> {
    let dir = wiki_dir();
    // Convert ID to file path (wiki:concepts:foo → concepts/foo.md)
    let file_path = dir.join(format!("{}.md", payload.id.replace(":", "/")));
    if !file_path.exists() {
        return Ok(serde_json::json!({ "success": false, "error": "Page not found" }));
    }
    let content = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    // Extract frontmatter + body
    if let Some((fm_str, body)) = content.split_once("---").and_then(|s| {
        s.1.find("---").map(|end| (&s.1[..end], &s.1[end+3..]))
    }) {
        if let Ok(meta) = serde_yaml::from_str::<WikiPageMeta>(fm_str) {
            return Ok(serde_json::json!({
                "success": true,
                "page": { "id": meta.id, "title": meta.title, "type": meta.page_type, "status": meta.status },
                "content": body.trim(),
            }));
        }
    }
    Ok(serde_json::json!({ "success": false, "error": "Invalid page format" }))
}

#[derive(Deserialize)]
pub struct CreatePagePayload {
    pub path: String,
    pub title: String,
    pub content: Option<String>,
    #[serde(rename = "type")]
    pub page_type: Option<String>,
}

#[tauri::command]
pub fn create_page(payload: CreatePagePayload) -> Result<Value, String> {
    let dir = wiki_dir();
    let file_path = dir.join(format!("{}.md", payload.path));
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = format!(
        "---\nid: wiki:{}\ntitle: {}\ntype: {}\nstatus: draft\n---\n\n{}",
        payload.path, payload.title, payload.page_type.unwrap_or_else(|| "concept".into()),
        payload.content.unwrap_or_default(),
    );
    std::fs::write(&file_path, &content).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "success": true, "id": format!("wiki:{}", payload.path) }))
}

// ─── Tasks ──────────────────────────────────────────

#[tauri::command]
pub fn task_board() -> Result<Value, String> {
    let dir = wiki_dir();
    let pages = read_all_pages(&dir);
    let tasks: Vec<&WikiPageMeta> = pages.iter().filter(|p| p.page_type.as_str() == "task").collect();
    let mut columns: std::collections::BTreeMap<String, Vec<Value>> = std::collections::BTreeMap::new();
    for task in &tasks {
        let status = task.status.to_string();
        columns.entry(status.clone()).or_default();
        columns.get_mut(&status).unwrap().push(serde_json::json!({
            "id": task.id, "title": task.title,
            "priority": task.priority.clone().map(|p| format!("{:?}", p).to_lowercase()).unwrap_or_else(|| "medium".into()),
        }));
    }
    let status_order = ["draft", "todo", "in-progress", "in-review", "done", "blocked", "on-hold", "urgent", "cancelled", "archived"];
    let ordered: Vec<Value> = status_order.iter().filter_map(|s| columns.remove(*s)).flatten().collect();
    let counts: std::collections::BTreeMap<String, usize> = columns.iter().map(|(k, v)| (k.clone(), v.len())).collect();
    Ok(serde_json::json!({ "tasks": ordered, "columns": columns, "counts": counts }))
}

// ─── Memory ─────────────────────────────────────────

#[derive(Deserialize)]
pub struct ListMemoryPayload {
    pub _layer: Option<String>,
    pub _status: Option<String>,
}

#[tauri::command]
pub fn list_memory(state: State<'_, EngineState>, _payload: ListMemoryPayload) -> Result<Value, String> {
    let entries = state.session_memory.iter().map(|e| serde_json::json!({
        "id": e.id, "title": e.title, "content": e.content,
        "tags": e.tags, "created_at": e.created_at, "updated_at": e.updated_at,
    })).collect::<Vec<_>>();
    Ok(serde_json::json!({ "success": true, "entries": entries }))
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
