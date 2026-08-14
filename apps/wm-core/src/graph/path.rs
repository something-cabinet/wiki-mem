use std::collections::{HashMap, HashSet, VecDeque};

use petgraph::stable_graph::StableGraph;
use petgraph::visit::EdgeRef;

use crate::engine::{GraphEdge, WikiPageMeta};

pub fn find_path(
    graph: &StableGraph<WikiPageMeta, GraphEdge>,
    _index: &HashMap<String, petgraph::stable_graph::NodeIndex>,
    start: petgraph::stable_graph::NodeIndex,
    end: petgraph::stable_graph::NodeIndex,
    max_depth: usize,
) -> Vec<(String, String, Option<String>)> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut parent: HashMap<
        petgraph::stable_graph::NodeIndex,
        (petgraph::stable_graph::NodeIndex, String),
    > = HashMap::new();
    visited.insert(start);
    queue.push_back((start, 0usize));
    let mut found = false;
    while let Some((current, depth)) = queue.pop_front() {
        if current == end {
            found = true;
            break;
        }
        if depth >= max_depth {
            continue;
        }
        for edge in crate::graph::edges_undirected(graph, current) {
            let neighbor = if edge.source() == current {
                edge.target()
            } else {
                edge.source()
            };
            if visited.insert(neighbor) {
                let edge_type = format!("{:?}", edge.weight().edge_type).to_lowercase();
                parent.insert(neighbor, (current, edge_type));
                queue.push_back((neighbor, depth.wrapping_add(1)));
            }
        }
    }
    if !found {
        return vec![];
    }
    let mut path = Vec::new();
    let mut current = end;
    while current != start {
        if let Some((prev, edge_type)) = parent.get(&current) {
            path.push((
                graph[current].id.clone(),
                graph[current].title.clone(),
                Some(edge_type.clone()),
            ));
            current = *prev;
        } else {
            break;
        }
    }
    path.push((graph[start].id.clone(), graph[start].title.clone(), None));
    path.reverse();
    path
}
