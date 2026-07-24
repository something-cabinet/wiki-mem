use crate::mcp::prelude::*;
use petgraph::visit::EdgeRef;



#[derive(Deserialize, JsonSchema)]
struct WmGraphNeighborsInput {
    #[schemars(description = "Page ID")]
    id: String,
    #[allow(dead_code)] // populated by serde, used for future filtering
    #[schemars(description = "Traversal depth")]
    depth: Option<i32>,
    #[allow(dead_code)] // populated by serde, used for future filtering
    #[schemars(description = "Filter by edge type")]
    edge_type: Option<String>,
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

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_typed(
        "wm_graph.neighbors",
        "Get typed edges from a page",
        move |input: WmGraphNeighborsInput| {
            let id = input.id;
            let query = input.query;

            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let index = &snapshot.1;

            let start = index
                .get(&id)
                .ok_or_else(|| ToolError::not_found("page", &id))?;
            let mut neighbors = Vec::new();

            for edge in graph.edges(*start) {
                let target = edge.target();
                let edge_type = edge.weight();
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
                    edge_type.priority() as f64 * (1.0 + title_match + tag_match + exact_title)
                } else {
                    edge_type.priority() as f64
                };

                neighbors.push(serde_json::json!({
                    "id": meta.id,
                    "title": meta.title,
                    "edge_type": format!("{:?}", edge_type).to_lowercase(),
                    "score": score,
                }));
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
                *type_counts.entry(type_name.to_string()).or_insert(0) += 1;
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
                    let degree = graph.edges(idx).count()
                        + graph.edges_directed(idx, petgraph::Direction::Incoming).count();
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
                        let edge_type = &graph[edge_idx];
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
                            "edge_type": format!("{:?}", edge_type).to_lowercase(),
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
            let depth = input.depth.unwrap_or(1).min(5) as usize;

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
                        "type": format!("{:?}", edge.weight()).to_lowercase(),
                    }));
                    if visited.insert(target) {
                        queue.push_back((target, d + 1));
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
            let max_depth = input.max_depth.unwrap_or(10) as usize;

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
}
