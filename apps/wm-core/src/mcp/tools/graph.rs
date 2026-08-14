use crate::mcp::prelude::*;
use petgraph::visit::EdgeRef;

#[derive(Deserialize, JsonSchema)]
struct WmGraphNeighborsSchema {
    #[serde(rename = "depth")]
    #[schemars(description = "Traversal depth")]
    _depth: Option<i32>,
    #[serde(rename = "edge_type")]
    #[schemars(description = "Filter by edge type")]
    _edge_type: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct WmGraphNeighborsInput {
    #[schemars(description = "Page ID or code node (file path, or file#symbol)")]
    id: String,
    #[serde(flatten)]
    _schema: WmGraphNeighborsSchema,
    #[schemars(description = "Optional text query to rank results")]
    query: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct WmGraphStatsInput {}

#[derive(Deserialize, JsonSchema)]
struct WmGraphFullInput {
    #[schemars(description = "Optional filter by page type")]
    page_type: Option<String>,
    #[schemars(description = "Include edge data in response")]
    include_edges: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
struct WmGraphSubgraphInput {
    #[schemars(description = "Center page ID")]
    center: String,
    #[schemars(description = "Max traversal depth")]
    depth: Option<i32>,
}

#[derive(Deserialize, JsonSchema)]
struct WmGraphPathInput {
    #[schemars(description = "Start page ID")]
    start: String,
    #[schemars(description = "End page ID")]
    end: String,
    #[schemars(description = "Max path depth")]
    max_depth: Option<i32>,
}

#[derive(Deserialize, JsonSchema)]
struct WmGraphAffectedInput {
    #[schemars(description = "Page ID or code node (file path, or file#symbol) to analyze")]
    node: String,
    #[schemars(description = "Max traversal depth (default 10)")]
    max_depth: Option<i32>,
}

/// Serialize one code neighbor entry (AC-2.2: typed code edges alongside wiki
/// edges, with source location + provenance).
#[cfg(feature = "code-intel")]
fn code_neighbor_json(
    node_id: &str,
    title: &str,
    kind: &str,
    edge: &wm_code_intel::services::graph_resolver::ResolvedCodeEdge,
    score: f64,
) -> serde_json::Value {
    serde_json::json!({
        "id": node_id,
        "title": title,
        "kind": kind,
        "edge_type": edge.edge_type,
        "provenance": edge.provenance.as_str(),
        "line": edge.line,
        "source_file": edge.source_file,
        "target_file": edge.target_file,
        "score": score,
    })
}

/// Neighbor JSON for the opposite endpoint of `edge` relative to `node_id`.
#[cfg(feature = "code-intel")]
fn code_neighbor_entries(
    cg: &wm_code_intel::services::graph_resolver::CodeEdgeGraph,
    node_id: &str,
    edges: Vec<&wm_code_intel::services::graph_resolver::ResolvedCodeEdge>,
    query: Option<&str>,
) -> Vec<serde_json::Value> {
    use wm_code_intel::services::graph_resolver::CodeNodeRef;
    let mut out = Vec::new();
    for edge in edges {
        let (neighbor_id, neighbor_kind): (String, &str) = if edge.source_node_id() == node_id {
            let tid = edge.target_node_id();
            if tid.is_empty() {
                continue;
            }
            let kind = if edge.target_symbol.is_some() {
                "symbol"
            } else {
                "file"
            };
            (tid, kind)
        } else {
            let sid = edge.source_node_id();
            if sid.is_empty() {
                continue;
            }
            let kind = if edge.source_symbol.is_some() {
                "symbol"
            } else {
                "file"
            };
            (sid, kind)
        };
        let node = CodeNodeRef::parse(&neighbor_id, cg);
        let title = node.title();

        let score = if let Some(q) = query {
            let q_lower = q.to_lowercase();
            if title.to_lowercase().contains(&q_lower) {
                f64::from(
                    wm_engine::models::edge_type_model::EdgeType::from_str_flexible(
                        &edge.edge_type,
                    )
                    .priority(),
                )
                .mul_add(1.0, 2.0)
            } else {
                f64::from(
                    wm_engine::models::edge_type_model::EdgeType::from_str_flexible(
                        &edge.edge_type,
                    )
                    .priority(),
                )
            }
        } else {
            f64::from(
                wm_engine::models::edge_type_model::EdgeType::from_str_flexible(&edge.edge_type)
                    .priority(),
            )
        };

        out.push(code_neighbor_json(
            &neighbor_id,
            &title,
            neighbor_kind,
            edge,
            score,
        ));
    }
    out
}

/// Collect typed code edges for a code node id (AC-2.2). Returns `None` when
/// the id does not resolve to a code node with edges.
#[cfg(feature = "code-intel")]
fn code_neighbors_for_id(
    cg: &wm_code_intel::services::graph_resolver::CodeEdgeGraph,
    id: &str,
    query: Option<&str>,
) -> Option<Vec<serde_json::Value>> {
    use std::collections::HashSet;
    use wm_code_intel::services::graph_resolver::{CodeNodeRef, ResolvedCodeEdge};
    let node = CodeNodeRef::parse(id, cg);

    let mut edges: Vec<&ResolvedCodeEdge> = Vec::new();
    match &node {
        CodeNodeRef::Symbol { file, symbol } => {
            edges.extend(cg.outgoing_from_symbol(file, symbol));
            edges.extend(cg.incoming_to_symbol(file, symbol));
            // File-level edges so traversal can leave the defining file.
            edges.extend(cg.outgoing_from_file(file));
            edges.extend(cg.incoming_to_file(file));
        }
        CodeNodeRef::File(file) => {
            edges.extend(cg.outgoing_from_file(file));
            edges.extend(cg.incoming_to_file(file));
        }
        CodeNodeRef::SymbolName(name) => {
            edges.extend(cg.edges_for_symbol_name(name));
        }
    }

    // Dedupe edges (a symbol edge also appears in its file's edge list).
    type EdgeKey = (
        String,
        String,
        Option<String>,
        String,
        Option<String>,
        usize,
    );
    let mut seen: HashSet<EdgeKey> = HashSet::new();
    edges.retain(|e| {
        seen.insert((
            e.edge_type.clone(),
            e.source_file.clone(),
            e.source_symbol.clone(),
            e.target_file.clone(),
            e.target_symbol.clone(),
            e.line,
        ))
    });

    let entries = code_neighbor_entries(cg, id, edges, query);
    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_typed(
        "wm_graph.neighbors",
        "Get typed edges from a page or code node (file path, or file#symbol). Wiki pages return wiki edges; code nodes return typed calls/imports/inherits edges with provenance and file:line.",
        move |input: WmGraphNeighborsInput| {
            let id = input.id;
            let query = input.query;

            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let index = &snapshot.1;

            let start = index.get(&id).copied();
            let mut neighbors = Vec::new();

            if let Some(start) = start {
                for edge in graph.edges(start) {
                    let target = edge.target();
                    let weight = edge.weight();
                    let meta = &graph[target];

                    let score = if let Some(ref q) = query {
                        let q_lower = q.to_lowercase();
                        let title_match = if meta.title.to_lowercase().contains(&q_lower) {
                            4.0
                        } else {
                            0.0
                        };
                        let tag_match = if meta
                            .tags
                            .iter()
                            .any(|t| t.to_lowercase().contains(&q_lower))
                        {
                            2.2
                        } else {
                            0.0
                        };
                        let exact_title = if meta.title.to_lowercase() == q_lower {
                            8.0
                        } else {
                            0.0
                        };
                        f64::from(weight.priority()) * (1.0 + title_match + tag_match + exact_title)
                    } else {
                        f64::from(weight.priority())
                    };

                    neighbors.push(serde_json::json!({
                        "id": meta.id,
                        "title": meta.title,
                        "edge_type": format!("{:?}", weight.edge_type).to_lowercase(),
                        "provenance": weight.provenance.as_str(),
                        "score": score,
                    }));
                }
            }

            // AC-2.2: merge typed code edges when the id is a code node.
            #[cfg(feature = "code-intel")]
            {
                let root = e
                    .project_root
                    .read()
                    .map_err(|_| ToolError::lock_poisoned("project_root"))?
                    .clone();
                if let Ok(Some(cg)) = crate::graph::load_code_graph(&root) {
                    if let Some(code_neighbors) = code_neighbors_for_id(&cg, &id, query.as_deref()) {
                        neighbors.extend(code_neighbors);
                    }
                }
            }

            if neighbors.is_empty() && start.is_none() {
                return Err(ToolError::not_found("page", &id));
            }

            neighbors.sort_by(|a, b| {
                let sa = a["score"].as_f64().unwrap_or(0.0);
                let sb = b["score"].as_f64().unwrap_or(0.0);
                sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
            });

            Ok(serde_json::json!({
                "id": id,
                "neighbors": neighbors,
                "total": neighbors.len(),
            }))
        },
    );

    let e = engine.clone();
    registry.register_typed(
        "wm_graph.stats",
        "Graph statistics (node/edge counts by type)",
        move |_input: WmGraphStatsInput| {
            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let mut type_counts: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();
            for idx in graph.node_indices() {
                let type_name = graph[idx].page_type.as_str();
                let counter = type_counts.entry(type_name.to_string()).or_insert(0);
                *counter = counter.wrapping_add(1);
            }
            Ok(serde_json::json!({
                "nodes": graph.node_count(),
                "edges": graph.edge_count(),
                "types": type_counts,
            }))
        },
    );

    let e = engine.clone();
    registry.register_typed(
        "wm_graph.full",
        "Full graph dump — all nodes and edges for visualization",
        move |input: WmGraphFullInput| {
            let snapshot = e.graph.load();
            let graph = &snapshot.0;

            let include_edges = input.include_edges.unwrap_or(true);

            let nodes: Vec<serde_json::Value> = graph
                .node_indices()
                .filter_map(|idx| {
                    let meta = &graph[idx];
                    if let Some(ref pt) = input.page_type {
                        if meta.page_type.as_str() != pt.as_str() {
                            return None;
                        }
                    }
                    let degree = graph.edges(idx).count().wrapping_add(
                        graph
                            .edges_directed(idx, petgraph::Direction::Incoming)
                            .count(),
                    );
                    Some(serde_json::json!({
                        "id": meta.id,
                        "title": meta.title,
                        "page_type": meta.page_type,
                        "degree": degree,
                    }))
                })
                .collect();

            let mut result = serde_json::json!({
                "success": true,
                "nodes": nodes,
                "node_count": nodes.len(),
            });

            if include_edges {
                let edges: Vec<serde_json::Value> = graph
                    .edge_indices()
                    .filter_map(|edge_idx| {
                        let (source, target) = graph.edge_endpoints(edge_idx)?;
                        let weight = &graph[edge_idx];
                        if let Some(ref pt) = input.page_type {
                            if graph[source].page_type.as_str() != pt.as_str()
                                && graph[target].page_type.as_str() != pt.as_str()
                            {
                                return None;
                            }
                        }
                        Some(serde_json::json!({
                            "source": graph[source].id,
                            "target": graph[target].id,
                            "edge_type": format!("{:?}", weight.edge_type).to_lowercase(),
                            "provenance": weight.provenance.as_str(),
                        }))
                    })
                    .collect();
                result["edges"] = serde_json::json!(edges);
                result["edge_count"] = serde_json::json!(edges.len());
            }

            Ok(result)
        },
    );

    let e = engine.clone();
    registry.register_typed(
        "wm_graph.subgraph",
        "Get neighborhood around a page node",
        move |input: WmGraphSubgraphInput| {
            let center = input.center;
            let depth = usize::try_from(input.depth.unwrap_or(1).min(5)).unwrap_or(5);

            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let index = &snapshot.1;

            let start = match index.get(&center) {
                Some(s) => *s,
                None => return Err(ToolError::not_found("page", &center)),
            };

            use std::collections::VecDeque;
            let mut visited = std::collections::HashSet::new();
            let mut queue = VecDeque::new();
            let mut nodes = Vec::new();
            let mut edges = Vec::new();

            visited.insert(start);
            queue.push_back((start, 0usize));

            while let Some((current, d)) = queue.pop_front() {
                if d > depth {
                    continue;
                }
                let meta = &graph[current];
                nodes.push(serde_json::json!({
                    "id": meta.id, "title": meta.title,
                    "type": meta.page_type.as_str(),
                    "depth": d,
                }));
                for edge in graph.edges(current) {
                    let target = edge.target();
                    edges.push(serde_json::json!({
                        "source": graph[current].id,
                        "target": graph[target].id,
                        "type": format!("{:?}", edge.weight().edge_type).to_lowercase(),
                        "provenance": edge.weight().provenance.as_str(),
                    }));
                    if visited.insert(target) {
                        queue.push_back((target, d.wrapping_add(1)));
                    }
                }
            }

            Ok(serde_json::json!({
                "center": center,
                "depth": depth,
                "nodes": nodes,
                "edges": edges,
                "node_count": nodes.len(),
            }))
        },
    );

    let e = engine.clone();
    registry.register_typed(
        "wm_graph.path",
        "Find shortest path between two pages",
        move |input: WmGraphPathInput| {
            let start_id = input.start;
            let end_id = input.end;
            let max_depth = usize::try_from(input.max_depth.unwrap_or(10)).unwrap_or(10);

            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let index = &snapshot.1;

            let start = index
                .get(&start_id)
                .ok_or_else(|| ToolError::not_found("page", &start_id))?;
            let end = index
                .get(&end_id)
                .ok_or_else(|| ToolError::not_found("page", &end_id))?;

            let path = crate::graph::find_path(graph, index, *start, *end, max_depth);

            if path.is_empty() {
                Ok(serde_json::json!({ "path": [], "length": 0, "note": "No path found" }))
            } else {
                let json_path: Vec<serde_json::Value> = path
                    .iter()
                    .map(|(id, title, edge_type)| {
                        serde_json::json!({
                            "id": id,
                            "title": title,
                            "edge_from_parent": edge_type,
                        })
                    })
                    .collect();
                Ok(serde_json::json!({ "path": json_path, "length": json_path.len() }))
            }
        },
    );

    let e = engine.clone();
    registry.register_typed(
        "wm_graph.affected",
        "Blast-radius analysis: nodes that break if the given node is removed. Wiki: incoming depends_on/extends. Code (file or file#symbol): incoming calls/imports/inherits. Returns each affected node with the edge path and per-hop provenance.",
        move |input: WmGraphAffectedInput| {
            let node_id = input.node;
            let max_depth = usize::try_from(input.max_depth.unwrap_or(10).min(25)).unwrap_or(10);

            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let index = &snapshot.1;

            // Wiki node.
            if let Some(&start_idx) = index.get(&node_id) {
                let affected = crate::graph::affected_wiki_nodes(graph, start_idx, max_depth);
                let items: Vec<serde_json::Value> = affected
                    .iter()
                    .map(|a| {
                        serde_json::json!({
                            "id": a.node_id,
                            "title": a.title,
                            "depth": a.depth(),
                            "hops": a.hops.iter().map(|h| serde_json::json!({
                                "edge_type": h.edge_type,
                                "from": h.from,
                                "to": h.to,
                                "line": h.line,
                                "provenance": h.provenance.as_str(),
                            })).collect::<Vec<_>>(),
                        })
                    })
                    .collect();
                return Ok(serde_json::json!({
                    "node": node_id,
                    "kind": "page",
                    "affected": items,
                    "total": items.len(),
                }));
            }

            // Code node.
            #[cfg(feature = "code-intel")]
            {
                let root = e
                    .project_root
                    .read()
                    .map_err(|_| ToolError::lock_poisoned("project_root"))?
                    .clone();
                if let Ok(Some(cg)) = crate::graph::load_code_graph(&root) {
                    use crate::graph::affected::code::affected_code_nodes;
                    use wm_code_intel::services::graph_resolver::CodeNodeRef;
                    let start = CodeNodeRef::parse(&node_id, &cg);
                    let has_edges = match &start {
                        CodeNodeRef::Symbol { file, symbol } => cg.has_symbol(file, symbol),
                        CodeNodeRef::File(file) => cg.has_file(file),
                        CodeNodeRef::SymbolName(name) => {
                            !cg.edges_for_symbol_name(name).is_empty()
                        }
                    };
                    if has_edges {
                        let affected = affected_code_nodes(&cg, &start, max_depth);
                        let items: Vec<serde_json::Value> = affected
                            .iter()
                            .map(|a| {
                                serde_json::json!({
                                    "id": a.node_id,
                                    "title": a.title,
                                    "depth": a.depth(),
                                    "hops": a.hops.iter().map(|h| serde_json::json!({
                                        "edge_type": h.edge_type,
                                        "from": h.from,
                                        "to": h.to,
                                        "line": h.line,
                                        "provenance": h.provenance.as_str(),
                                    })).collect::<Vec<_>>(),
                                })
                            })
                            .collect();
                        return Ok(serde_json::json!({
                            "node": node_id,
                            "kind": "code",
                            "affected": items,
                            "total": items.len(),
                        }));
                    }
                }
            }

            Err(ToolError::not_found("page", &node_id))
        },
    );
}
