use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::Graph;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
struct GraphInputNode {
    id: String,
}

#[derive(Deserialize)]
struct GraphInputEdge {
    source: String,
    target: String,
    #[serde(rename = "edgeType", default)]
    edge_type: String,
}

#[derive(Serialize)]
struct PathResult {
    ids: Vec<String>,
}

#[derive(Serialize)]
struct NeighborResult {
    id: String,
    edge_type: String,
}

#[wasm_bindgen]
pub struct GraphAlgo {
    graph: Graph<String, String>,
    id_to_index: std::collections::HashMap<String, NodeIndex>,
}

#[wasm_bindgen]
impl GraphAlgo {
    pub fn new(nodes_json: &str, edges_json: &str) -> Result<GraphAlgo, JsValue> {
        let nodes: Vec<GraphInputNode> =
            serde_json::from_str(nodes_json).map_err(|e| e.to_string())?;
        let edges: Vec<GraphInputEdge> =
            serde_json::from_str(edges_json).map_err(|e| e.to_string())?;

        let mut graph = Graph::<String, String>::new();
        let mut id_to_index = std::collections::HashMap::new();

        for node in &nodes {
            let idx = graph.add_node(node.id.clone());
            id_to_index.insert(node.id.clone(), idx);
        }

        for edge in &edges {
            if let (Some(&source), Some(&target)) =
                (id_to_index.get(&edge.source), id_to_index.get(&edge.target))
            {
                graph.add_edge(source, target, edge.edge_type.clone());
            }
        }

        Ok(GraphAlgo { graph, id_to_index })
    }

    pub fn find_path(&self, start_id: &str, end_id: &str) -> Result<String, JsValue> {
        use petgraph::algo::has_path_connecting;

        let start = self
            .id_to_index
            .get(start_id)
            .ok_or("Start node not found")?;
        let end = self.id_to_index.get(end_id).ok_or("End node not found")?;

        if !has_path_connecting(&self.graph, *start, *end, None) {
            return Ok(serde_json::to_string(&PathResult { ids: vec![] }).unwrap());
        }

        use petgraph::visit::Bfs;
        let mut bfs = Bfs::new(&self.graph, *start);
        let mut parent = std::collections::HashMap::new();

        while let Some(nx) = bfs.next(&self.graph) {
            if nx == *end {
                break;
            }
            for edge in self.graph.edges(nx) {
                let target = edge.target();
                if !parent.contains_key(&target) && target != *start {
                    parent.insert(target, nx);
                }
            }
        }

        let mut path = Vec::new();
        let mut current = *end;
        while current != *start {
            path.push(self.graph[current].clone());
            current = *parent.get(&current).ok_or("Path reconstruction failed")?;
        }
        path.push(self.graph[*start].clone());
        path.reverse();

        Ok(serde_json::to_string(&PathResult { ids: path }).unwrap())
    }

    pub fn neighbors(&self, node_id: &str) -> Result<String, JsValue> {
        let idx = self.id_to_index.get(node_id).ok_or("Node not found")?;
        let mut result = Vec::new();

        for edge in self.graph.edges(*idx) {
            let neighbor_id = if edge.target() == *idx {
                &self.graph[edge.source()]
            } else {
                &self.graph[edge.target()]
            };
            result.push(NeighborResult {
                id: neighbor_id.clone(),
                edge_type: edge.weight().clone(),
            });
        }

        for edge in self
            .graph
            .edges_directed(*idx, petgraph::Direction::Incoming)
        {
            let neighbor_id = &self.graph[edge.source()];
            if !result.iter().any(|r| r.id == *neighbor_id) {
                result.push(NeighborResult {
                    id: neighbor_id.clone(),
                    edge_type: edge.weight().clone(),
                });
            }
        }

        Ok(serde_json::to_string(&result).unwrap())
    }

    pub fn subgraph(&self, center_id: &str, depth: usize) -> Result<String, JsValue> {
        let center = self
            .id_to_index
            .get(center_id)
            .ok_or("Center node not found")?;
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        let mut distances = std::collections::HashMap::new();

        visited.insert(*center);
        distances.insert(*center, 0usize);
        queue.push_back(*center);

        while let Some(current) = queue.pop_front() {
            let dist = distances[&current];
            if dist >= depth {
                continue;
            }
            for edge in self.graph.edges(current) {
                let target = edge.target();
                if visited.insert(target) {
                    distances.insert(target, dist + 1);
                    queue.push_back(target);
                }
            }
            for edge in self
                .graph
                .edges_directed(current, petgraph::Direction::Incoming)
            {
                let source = edge.source();
                if visited.insert(source) {
                    distances.insert(source, dist + 1);
                    queue.push_back(source);
                }
            }
        }

        let mut sub_nodes: Vec<String> = Vec::new();
        let mut sub_edges: Vec<serde_json::Value> = Vec::new();

        for &idx in &visited {
            sub_nodes.push(self.graph[idx].clone());
        }

        for edge in self.graph.edge_references() {
            if visited.contains(&edge.source()) && visited.contains(&edge.target()) {
                sub_edges.push(serde_json::json!({
                    "source": self.graph[edge.source()],
                    "target": self.graph[edge.target()],
                    "edgeType": edge.weight()
                }));
            }
        }

        Ok(serde_json::to_string(&serde_json::json!({
            "nodes": sub_nodes,
            "edges": sub_edges
        }))
        .unwrap())
    }
}
