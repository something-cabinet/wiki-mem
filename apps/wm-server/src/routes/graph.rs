use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use petgraph::visit::EdgeRef;
use serde::Deserialize;
use serde_json::{json, Value};

pub async fn stats(State(state): State<Arc<wm_core::engine::EngineState>>) -> Json<Value> {
    let snapshot = state.graph.load();
    let graph = &snapshot.0;
    let mut node_count = 0i64;
    let mut type_counts: HashMap<String, i64> = HashMap::new();

    for node in graph.node_indices() {
        node_count = node_count.wrapping_add(1);
        let meta = &graph[node];
        type_counts
            .entry(meta.page_type.as_str().to_string())
            .and_modify(|v| *v = v.wrapping_add(1))
            .or_insert(1);
    }
    let edge_count = i64::try_from(graph.edge_count()).unwrap_or(0);

    Json(json!({
        "success": true,
        "node_count": node_count,
        "edge_count": edge_count,
        "type_counts": type_counts,
    }))
}

pub async fn full(State(state): State<Arc<wm_core::engine::EngineState>>) -> Json<Value> {
    let snapshot = state.graph.load();
    let graph = &snapshot.0;
    let id_index = &snapshot.1;

    let nodes: Vec<Value> = graph
        .node_indices()
        .map(|i| {
            let meta = &graph[i];
            json!({
                "id": meta.id,
                "title": meta.title,
                "page_type": meta.page_type,
                "degree": wm_core::graph::edges_undirected(graph, i).len(),
            })
        })
        .collect();

    let edges: Vec<Value> = graph
        .edge_indices()
        .filter_map(|i| {
            let (source, target) = graph.edge_endpoints(i)?;
            let edge = &graph[i];
            let source_id = id_index
                .iter()
                .find(|(_, &idx)| idx == source)
                .map(|(id, _)| id.clone());
            let target_id = id_index
                .iter()
                .find(|(_, &idx)| idx == target)
                .map(|(id, _)| id.clone());
            Some(json!({
                "source": source_id.unwrap_or_default(),
                "target": target_id.unwrap_or_default(),
                "edge_type": format!("{:?}", edge.edge_type).to_lowercase(),
                "provenance": edge.provenance.as_str(),
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
            let items: Vec<Value> = wm_core::graph::edges_undirected(graph, node_idx)
                .into_iter()
                .map(|edge| {
                    let neighbor = if edge.source() == node_idx {
                        edge.target()
                    } else {
                        edge.source()
                    };
                    let meta = &graph[neighbor];
                    let weight = edge.weight();
                    json!({
                        "id": meta.id,
                        "title": meta.title,
                        "page_type": meta.page_type,
                        "edge_type": format!("{:?}", weight.edge_type).to_lowercase(),
                        "provenance": weight.provenance.as_str(),
                    })
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
            let result = wm_core::graph::find_path(graph, id_index, start_idx, end_idx, max_depth);
            let path: Vec<Value> = result
                .into_iter()
                .map(|(id, title, _edge_type)| json!({"id": id, "title": title}))
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

#[derive(Deserialize)]
pub struct AffectedInput {
    pub node: String,
    pub max_depth: Option<usize>,
}

/// Blast-radius analysis: transitive breakage set for a wiki page or
/// code node, with per-hop provenance and (for code) file:line.
pub async fn affected(
    State(state): State<Arc<wm_core::engine::EngineState>>,
    Json(input): Json<AffectedInput>,
) -> Json<Value> {
    let max_depth = input.max_depth.unwrap_or(10).min(25);
    let snapshot = state.graph.load();
    let graph = &snapshot.0;
    let index = &snapshot.1;

    if let Some(&start_idx) = index.get(&input.node) {
        let affected = wm_core::graph::affected_wiki_nodes(graph, start_idx, max_depth);
        let items: Vec<Value> = affected
            .iter()
            .map(|a| {
                json!({
                    "id": a.node_id,
                    "title": a.title,
                    "depth": a.depth(),
                    "hops": a.hops.iter().map(|h| json!({
                        "edge_type": h.edge_type,
                        "from": h.from,
                        "to": h.to,
                        "line": h.line,
                        "provenance": h.provenance.as_str(),
                    })).collect::<Vec<_>>(),
                })
            })
            .collect();
        return Json(json!({
            "success": true,
            "node": input.node,
            "kind": "page",
            "affected": items,
            "total": items.len(),
        }));
    }

    {
        let root = state
            .project_root
            .read()
            .map(|r| r.clone())
            .unwrap_or_default();
        if let Ok(Some(cg)) = wm_core::graph::load_code_graph(&root) {
            use wm_core::code_intel::services::graph_resolver::CodeNodeRef;
            use wm_core::graph::affected::code::affected_code_nodes;
            let start = CodeNodeRef::parse(&input.node, &cg);
            let has_edges = match &start {
                CodeNodeRef::Symbol { file, symbol } => cg.has_symbol(file, symbol),
                CodeNodeRef::File(file) => cg.has_file(file),
                CodeNodeRef::SymbolName(name) => !cg.edges_for_symbol_name(name).is_empty(),
            };
            if has_edges {
                let affected = affected_code_nodes(&cg, &start, max_depth);
                let items: Vec<Value> = affected
                    .iter()
                    .map(|a| {
                        json!({
                            "id": a.node_id,
                            "title": a.title,
                            "depth": a.depth(),
                            "hops": a.hops.iter().map(|h| json!({
                                "edge_type": h.edge_type,
                                "from": h.from,
                                "to": h.to,
                                "line": h.line,
                                "provenance": h.provenance.as_str(),
                            })).collect::<Vec<_>>(),
                        })
                    })
                    .collect();
                return Json(json!({
                    "success": true,
                    "node": input.node,
                    "kind": "code",
                    "affected": items,
                    "total": items.len(),
                }));
            }
        }
    }

    Json(json!({
        "success": true,
        "node": input.node,
        "affected": [],
        "total": 0,
    }))
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
            use std::collections::VecDeque;
            let mut visited = std::collections::HashSet::new();
            let mut queue = VecDeque::new();
            let mut node_ids = std::collections::HashSet::new();

            visited.insert(center_idx);
            queue.push_back((center_idx, 0usize));

            while let Some((current, d)) = queue.pop_front() {
                if d > depth || node_ids.len() > 100 {
                    continue;
                }
                node_ids.insert(current);
                for edge in wm_core::graph::edges_undirected(graph, current) {
                    let neighbor = if edge.source() == current {
                        edge.target()
                    } else {
                        edge.source()
                    };
                    if visited.insert(neighbor) {
                        queue.push_back((neighbor, d.wrapping_add(1)));
                    }
                }
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
                            "edge_type": format!("{:?}", edge.edge_type).to_lowercase(),
                            "provenance": edge.provenance.as_str(),
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
