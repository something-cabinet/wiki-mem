use petgraph::visit::EdgeRef;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};
use tauri::{State, AppHandle, Emitter};
use wm_core::engine::{EngineState, WikiPageMeta};
use wm_core::search::{self, QueryParams};

// Debug event buffer for pilot tests (D6)
#[cfg(debug_assertions)]
static CAPTURED_EVENTS: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(Vec::new()));

// ─── Initial ─────────────────────────────────────────

#[tauri::command]
pub fn get_initial(state: State<'_, Arc<EngineState>>) -> Result<Value, String> {
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
pub fn search(state: State<'_, Arc<EngineState>>, payload: SearchPayload) -> Result<Value, String> {
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

fn detect_project_root() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut max_depth = 20;
    loop {
        if dir.join(".wm").join("config.json").exists() {
            return dir;
        }
        if max_depth == 0 || !dir.pop() {
            return std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        }
        max_depth -= 1;
    }
}

fn wiki_dir() -> PathBuf {
    detect_project_root().join(".wm").join("wiki")
}

fn read_all_pages(dir: &PathBuf) -> Vec<WikiPageMeta> {
    if !dir.exists() { return vec![]; }
    let mut pages = Vec::new();
    collect_md_files_recursive(dir, dir, &mut pages);
    pages
}

#[derive(Deserialize)]
struct SimplePageMeta {
    #[serde(default)]
    title: String,
    #[serde(default, alias = "type")]
    page_type: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    id: String,
    #[serde(default)]
    tags: Vec<String>,
}

fn parse_page_type(s: &str) -> wm_core::engine::PageType {
    match s.to_lowercase().as_str() {
        "task" => wm_core::engine::PageType::Task,
        "spec" => wm_core::engine::PageType::Spec,
        "concept" => wm_core::engine::PageType::Concept,
        "pattern" => wm_core::engine::PageType::Pattern,
        "decision" => wm_core::engine::PageType::Decision,
        "memory" => wm_core::engine::PageType::Memory,
        "howto" | "guide" => wm_core::engine::PageType::Howto,
        "reference" => wm_core::engine::PageType::Reference,
        "note" | "notes" => wm_core::engine::PageType::Note,
        "rule" => wm_core::engine::PageType::Rule,
        _ => wm_core::engine::PageType::Concept,
    }
}

fn collect_md_files_recursive(wiki_root: &PathBuf, dir: &PathBuf, pages: &mut Vec<WikiPageMeta>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_md_files_recursive(wiki_root, &path, pages);
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                let content = std::fs::read_to_string(&path).unwrap_or_default();
                if let Some((fm, _)) = content.split_once("---").and_then(|s| {
                    s.1.find("---").map(|end| (&s.1[..end], &s.1[end+3..]))
                }) {
                    if let Ok(meta) = serde_yaml::from_str::<WikiPageMeta>(fm) {
                        pages.push(meta);
                    } else if let Ok(simple) = serde_yaml::from_str::<SimplePageMeta>(fm) {
                        pages.push(WikiPageMeta {
                            id: if simple.id.is_empty() {
                                path.strip_prefix(wiki_root)
                                    .unwrap_or(&path)
                                    .with_extension("")
                                    .to_string_lossy()
                                    .to_string()
                                    .replace('\\', "/")
                            } else { simple.id },
                            title: simple.title,
                            tags: simple.tags,
                            status: wm_core::engine::PageStatus::Draft,
                            published: false,
                            priority: None,
                            confidence: None,
                            assignee: None,
                            aliases: vec![],
                            superseded_by: None,
                            version: None,
                            sources: vec![],
                            parent: None,
                            relates_to: vec![],
                            path: path.clone(),
                            created_at: String::new(),
                            updated_at: String::new(),
                            page_type: parse_page_type(&simple.page_type),
                            order: None,
                            task_data: None,
                            spec_data: None,
                            decision_data: None,
                            pattern_data: None,
                            memory_data: None,
                            rule_data: None,
                        });
                    }
                }
            }
        }
    }
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
    // Strip wiki: prefix then convert to file path
    let clean_id = payload.id.strip_prefix("wiki:").unwrap_or(&payload.id);
    let file_path = dir.join(format!("{}.md", clean_id.replace(":", "/")));
    if !file_path.exists() {
        return Ok(serde_json::json!({ "success": false, "error": "Page not found" }));
    }
    let content = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    // Extract frontmatter + body
    if let Some((fm_str, body)) = content.split_once("---").and_then(|s| {
        s.1.find("---").map(|end| (&s.1[..end], &s.1[end+3..]))
    }) {
        let meta = serde_yaml::from_str::<WikiPageMeta>(fm_str).ok()
            .or_else(|| serde_yaml::from_str::<SimplePageMeta>(fm_str).ok().map(|s| WikiPageMeta {
                id: s.id, title: s.title, tags: s.tags,
                status: wm_core::engine::PageStatus::Draft, published: false,
                priority: None, confidence: None, assignee: None, aliases: vec![],
                superseded_by: None, version: None, sources: vec![], parent: None,
                relates_to: vec![], path: file_path.clone(), created_at: String::new(),
                updated_at: String::new(), page_type: parse_page_type(&s.page_type),
                order: None, task_data: None, spec_data: None, decision_data: None,
                pattern_data: None, memory_data: None, rule_data: None,
            }));
        if let Some(meta) = meta {
            return Ok(serde_json::json!({
                "success": true,
                "title": meta.title,
                "type": meta.page_type,
                "status": meta.status,
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
    pub tags: Option<String>,
}

#[tauri::command]
pub fn create_page(payload: CreatePagePayload) -> Result<Value, String> {
    let dir = wiki_dir();
    let file_path = dir.join(format!("{}.md", payload.path));
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tags_line = payload.tags.as_ref().map(|t| format!("\ntags: {}", t)).unwrap_or_default();
    let content = format!(
        "---\nid: wiki:{}\ntitle: {}\ntype: {}\nstatus: draft{}---\n\n{}",
        payload.path, payload.title, payload.page_type.unwrap_or_else(|| "concept".into()),
        tags_line,
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
    let counts: std::collections::BTreeMap<String, usize> = columns.iter().map(|(k, v)| (k.clone(), v.len())).collect();
    Ok(serde_json::json!({ "success": true, "columns": columns, "counts": counts }))
}

// ─── Memory ─────────────────────────────────────────

#[derive(Deserialize)]
pub struct ListMemoryPayload {
    pub _layer: Option<String>,
    pub _status: Option<String>,
}

#[tauri::command]
pub fn list_memory(state: State<'_, Arc<EngineState>>, _payload: ListMemoryPayload) -> Result<Value, String> {
    let entries = state.session_memory.iter().map(|e| serde_json::json!({
        "id": e.id, "title": e.title, "content": e.content,
        "tags": e.tags, "created_at": e.created_at, "updated_at": e.updated_at,
    })).collect::<Vec<_>>();
    Ok(serde_json::json!({ "success": true, "entries": entries }))
}

// ─── Graph ───────────────────────────────────────────

#[tauri::command]
pub fn get_graph_full(state: State<'_, Arc<EngineState>>) -> Result<Value, String> {
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
pub fn get_graph_stats(state: State<'_, Arc<EngineState>>) -> Result<Value, String> {
    let snapshot = state.graph.load();
    let graph = &snapshot.0;
    Ok(serde_json::json!({ "success": true, "node_count": graph.node_count(), "edge_count": graph.edge_count() }))
}

#[derive(Deserialize)]
pub struct NeighborPayload {
    pub id: String,
}

#[tauri::command]
pub fn get_graph_neighbors(state: State<'_, Arc<EngineState>>, payload: NeighborPayload) -> Result<Value, String> {
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

// ─── Page CRUD (REST-style) ─────────────────────

#[derive(Deserialize)]
pub struct UpdatePagePayload {
    pub id: String,
    pub path: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    #[serde(rename = "type")]
    pub page_type: Option<String>,
    pub tags: Option<String>,
    pub status: Option<String>,
}

#[tauri::command]
pub fn update_page(payload: UpdatePagePayload) -> Result<Value, String> {
    let dir = wiki_dir();
    let file_path = dir.join(format!("{}.md", payload.id.replace(":", "/")));
    if !file_path.exists() {
        return Ok(serde_json::json!({ "success": false, "error": "Page not found" }));
    }
    let content = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    // Replace frontmatter fields
    let (fm_str, body) = content.split_once("---").and_then(|s| {
        s.1.find("---").map(|end| (&s.1[..end], &s.1[end+3..]))
    }).ok_or_else(|| "Invalid page format".to_string())?;
    
    let mut fm_lines: Vec<String> = fm_str.lines().map(|l| l.to_string()).collect();
    
    // Helper to set or add a frontmatter field
    let mut set_field = |key: &str, value: &str| {
        let line = format!("{}: {}", key, value);
        if let Some(pos) = fm_lines.iter().position(|l| l.starts_with(&format!("{}:", key))) {
            fm_lines[pos] = line;
        } else {
            fm_lines.push(line);
        }
    };
    
    if let Some(ref v) = payload.title { set_field("title", v); }
    if let Some(ref v) = payload.page_type { set_field("type", v); }
    if let Some(ref v) = payload.status { set_field("status", v); }
    if let Some(ref v) = payload.tags { set_field("tags", v); }
    if let Some(ref v) = payload.path { set_field("id", &format!("wiki:{}", v)); }
    
    let new_fm = fm_lines.join("\n");
    let body_content = payload.content.as_deref().unwrap_or(body.trim());
    let new_content = format!("---\n{}---\n\n{}", new_fm, body_content);
    std::fs::write(&file_path, &new_content).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "success": true, "id": payload.id }))
}

#[derive(Deserialize)]
pub struct DeletePagePayload {
    pub id: String,
}

#[tauri::command]
pub fn delete_page(payload: DeletePagePayload) -> Result<Value, String> {
    let dir = wiki_dir();
    let file_path = dir.join(format!("{}.md", payload.id.replace(":", "/")));
    if !file_path.exists() {
        return Ok(serde_json::json!({ "success": false, "error": "Page not found" }));
    }
    std::fs::remove_file(&file_path).map_err(|e| e.to_string())?;
    // Also remove parent directory if empty
    if let Some(parent) = file_path.parent() {
        let _ = std::fs::remove_dir(parent);
    }
    Ok(serde_json::json!({ "success": true, "id": payload.id }))
}

// ─── Debug (pilot tests) ────────────────────────

#[cfg(debug_assertions)]
#[tauri::command]
pub fn get_captured_events() -> Result<Value, String> {
    let buf = CAPTURED_EVENTS.lock().unwrap();
    Ok(serde_json::json!({ "events": buf.clone() }))
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn clear_captured_events() -> Result<Value, String> {
    let mut buf = CAPTURED_EVENTS.lock().unwrap();
    buf.clear();
    Ok(serde_json::json!({ "success": true }))
}

// ─── fjadra Force Layout ────────────────────────

#[derive(Deserialize)]
pub struct LayoutNode {
    #[allow(dead_code)] // populated by serde, used by ComputeLayoutPayload
    pub id: String,
    pub x: Option<f64>,
    pub y: Option<f64>,
}

#[derive(Deserialize)]
pub struct LayoutEdge {
    pub source: usize,
    pub target: usize,
}

#[derive(Deserialize)]
pub struct ComputeLayoutPayload {
    pub nodes: Vec<LayoutNode>,
    pub edges: Vec<LayoutEdge>,
    #[serde(default = "default_viewport")]
    pub width: f64,
    #[serde(default = "default_viewport")]
    pub height: f64,
}

fn default_viewport() -> f64 { 800.0 }

#[tauri::command]
pub async fn compute_layout(
    app: AppHandle,
    payload: ComputeLayoutPayload,
) -> Result<Value, String> {
    let n = payload.nodes.len();
    if n == 0 {
        return Ok(serde_json::json!({ "success": false, "error": "No nodes to layout" }));
    }

    // Build fjadra nodes — Node::default() for each, optionally with initial position
    let nodes: Vec<fjadra::force::Node> = payload.nodes.iter().map(|n| {
        match (n.x, n.y) {
            (Some(x), Some(y)) => fjadra::force::Node::default().position(x, y),
            _ => fjadra::force::Node::default(),
        }
    }).collect();

    // Build links from edge indices as (usize, usize) tuples
    let links: Vec<(usize, usize)> = payload.edges.iter().map(|e| (e.source, e.target)).collect();

    // Build and run simulation
    let mut sim = fjadra::force::SimulationBuilder::new()
        .build(nodes)
        .add_force("charge", fjadra::force::ManyBody::new().strength(-200.0))
        .add_force("link", fjadra::force::Link::new(links).distance(80.0).strength(0.3))
        .add_force("center", fjadra::force::Center::new().x(payload.width / 2.0).y(payload.height / 2.0))
        .add_force("collide", fjadra::force::Collide::new());

    let max_ticks = 300;
    let coarse_ticks = 30;

    for tick in 0..max_ticks {
        sim.tick(1);
        let positions: Vec<[f64; 2]> = sim.positions().collect();

        // Yield every 10 ticks to avoid blocking the Tauri async runtime
        if tick % 10 == 0 {
            tokio::task::yield_now().await;
        }

        if tick == coarse_ticks - 1 {
            let _ = app.emit("graph-coarse", serde_json::json!({ "positions": positions }));
        }

        if tick >= coarse_ticks && tick % 10 == 0 {
            let _ = app.emit("graph-refine", serde_json::json!({ "positions": positions, "tick": tick }));
        }

        if sim.is_finished() {
            break;
        }
    }

    // Final positions
    let positions: Vec<[f64; 2]> = sim.positions().collect();
    let _ = app.emit("graph-settled", serde_json::json!({ "positions": positions }));

    #[cfg(debug_assertions)]
    {
        let mut buf = CAPTURED_EVENTS.lock().unwrap();
        buf.push("graph-coarse".to_string());
        buf.push("graph-refine".to_string());
        buf.push("graph-settled".to_string());
    }

    Ok(serde_json::json!({
        "success": true,
        "ticks": max_ticks.min(300),
        "nodes": n,
        "edges": payload.edges.len(),
    }))
}
