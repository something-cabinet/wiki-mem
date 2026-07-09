use petgraph::visit::EdgeRef;
use std::sync::Arc;

use serde_json::json;

use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;

/// Register graph tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_with_schema(
        "graph.neighbors",
        "Get typed edges from a page",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Page ID" },
                "depth": { "type": "integer", "description": "Traversal depth" },
                "edge_type": { "type": "string", "description": "Filter by edge type" }
            },
            "required": ["id"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let query = args.optional_string("query");

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
        }),
    );

    let e = engine.clone();
    registry.register_with_schema(
        "graph.stats",
        "Graph statistics (node/edge counts by type)",
        json!({
            "type": "object",
            "properties": {}
        }),
        Arc::new(move |_params| {
            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let mut type_counts: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();
            for idx in graph.node_indices() {
                let type_name = format!("{:?}", graph[idx].page_type).to_lowercase();
                *type_counts.entry(type_name).or_insert(0) += 1;
            }
            Ok(serde_json::json!({
                "nodes": graph.node_count(),
                "edges": graph.edge_count(),
                "types": type_counts,
            }))
        }),
    );

    let e = engine.clone();
    registry.register_with_schema(
        "graph.subgraph",
        "Get neighborhood around a page node",
        json!({
            "type": "object",
            "properties": {
                "center": { "type": "string", "description": "Center page ID" },
                "depth": { "type": "integer", "description": "Max traversal depth", "default": 1 }
            },
            "required": ["center"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let center = args.require_string("center")?;
            let depth = args.optional_int("depth").unwrap_or(1).min(5);

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
                    "type": format!("{:?}", meta.page_type).to_lowercase(),
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
        }),
    );

    let e = engine.clone();
    registry.register_with_schema(
        "graph.path",
        "Find shortest path between two pages",
        json!({
            "type": "object",
            "properties": {
                "start": { "type": "string", "description": "Start page ID" },
                "end": { "type": "string", "description": "End page ID" },
                "max_depth": { "type": "integer", "description": "Max path depth", "default": 10 }
            },
            "required": ["start", "end"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let start_id = args.require_string("start")?;
            let end_id = args.require_string("end")?;
            let max_depth = args.optional_int("max_depth").unwrap_or(10) as usize;

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
        }),
    );
}
