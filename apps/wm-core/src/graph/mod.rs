use petgraph::stable_graph::StableGraph;
use std::collections::HashMap;
use std::path::Path;
use tracing::info;

use crate::engine::{EdgeType, GraphSnapshot, WikiPageMeta};
use crate::parser::parse_edge_type;

pub mod sections;
pub mod index_gen;
pub mod lint;
pub mod path;

pub use sections::*;
pub use index_gen::*;
pub use lint::*;
pub use path::*;

pub fn rebuild_graph_snapshot(
    graph_swap: &arc_swap::ArcSwap<GraphSnapshot>,
    wiki_dir: &Path,
    custom_types: &[String],
) -> usize {
    let (graph, id_index) = build_graph_from_wiki(wiki_dir, custom_types);
    let node_count = graph.node_count();
    let snapshot = std::sync::Arc::new((graph, id_index));
    graph_swap.store(snapshot);
    info!("Graph rebuilt: {} nodes, replaced atomically.", node_count);
    node_count
}

struct ParsedPage {
    meta: WikiPageMeta,
    edges: Vec<(EdgeType, String)>,
    custom_types: Vec<String>,
}

pub fn build_graph_from_wiki(
    wiki_dir: &Path,
    registered_custom_types: &[String],
) -> (
    StableGraph<WikiPageMeta, EdgeType>,
    HashMap<String, petgraph::stable_graph::NodeIndex>,
) {
    use petgraph::algo::is_cyclic_directed;
    use rayon::prelude::*;
    use tracing::warn;

    let mut graph = StableGraph::<WikiPageMeta, EdgeType>::new();
    let mut id_index = HashMap::new();

    let paths: Vec<_> = walkdir::WalkDir::new(wiki_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
        .filter(|e| {
            let name = e.file_name().to_string_lossy();
            name != "index.md" && name != "log.md"
        })
        .map(|e| (e.path().to_path_buf(), e.path().to_path_buf()))
        .collect();

    use crate::parser::parse_wiki_page;
    let parsed: Vec<ParsedPage> = paths
        .par_iter()
        .filter_map(|(path, _)| {
            let wiki_rel = path.strip_prefix(wiki_dir).unwrap_or(path);
            let rel_path = Path::new(".wm").join("wiki").join(wiki_rel);
            let content = std::fs::read_to_string(path).ok()?;
            if content.trim().is_empty() {
                return None;
            }
            let meta = parse_wiki_page(&rel_path, &content);
            let mut edges: Vec<(EdgeType, String)> = Vec::new();
            let mut custom_types: Vec<String> = Vec::new();
            for (edge_type, target) in &meta.relates_to {
                let edge_type_str = match edge_type {
                    EdgeType::Custom(name) => name.to_lowercase(),
                    _ => format!("{:?}", edge_type).to_lowercase(),
                };
                edges.push((edge_type.clone(), target.clone()));
                if is_custom_edge(&edge_type_str)
                    && !custom_types.contains(&edge_type_str)
                {
                    custom_types.push(edge_type_str);
                }
            }
            Some(ParsedPage { meta, edges, custom_types })
        })
        .collect();

    let mut pending_edges: Vec<(String, EdgeType, String)> = Vec::new();
    let mut used_custom_types: Vec<String> = Vec::new();

    for page in parsed {
        let node_idx = graph.add_node(page.meta.clone());
        id_index.insert(page.meta.id.clone(), node_idx);
        for (edge_type, target) in page.edges {
            pending_edges.push((page.meta.id.clone(), edge_type, target));
        }
        for ct in page.custom_types {
            if !used_custom_types.contains(&ct) {
                used_custom_types.push(ct);
            }
        }
    }

    let rejected = validate_custom_edge_types(registered_custom_types, &used_custom_types);
    for t in &rejected {
        warn!(
            "Custom edge type '{}' not registered in config. Skipping edges of this type.",
            t
        );
    }

    let mut added_edges: std::collections::HashSet<(
        petgraph::stable_graph::NodeIndex,
        petgraph::stable_graph::NodeIndex,
    )> = std::collections::HashSet::new();
    for (source_id, edge_type, target) in &pending_edges {
        let edge_type_str = match edge_type {
            EdgeType::Custom(name) => name.to_lowercase(),
            _ => format!("{:?}", edge_type).to_lowercase(),
        };
        if rejected.contains(&edge_type_str) {
            continue;
        }
        if let Some(&source_idx) = id_index.get(source_id) {
            // Normalize target ID: replace / with : to match path_to_id format
            let normalized_target = target.replace('/', ":");
            let target_idx = id_index.get(&normalized_target).copied().or_else(|| {
                id_index.get(target).copied().or_else(|| {
                    crate::parser::resolve_link_target(target, &graph)
                        .and_then(|id| id_index.get(&id).copied())
                })
            });
            if target_idx.is_none() {
                tracing::debug!("Graph: unresolved relates_to target '{}' from '{}'", target, source_id);
            }
            if let Some(target_idx) = target_idx {
                if added_edges.insert((source_idx, target_idx)) {
                    graph.add_edge(source_idx, target_idx, edge_type.clone());
                }
            }
        }
    }

    if is_cyclic_directed(&graph) {
        info!("Cycle detected in wiki graph (expected: mutual relates_to links). BFS uses visited tracking to prevent infinite loops.");
    } else {
        info!("Graph is acyclic — safe for topological operations.");
    }

    (graph, id_index)
}

pub fn validate_custom_edge_types(registered: &[String], used_types: &[String]) -> Vec<String> {
    let mut rejected = Vec::new();
    for t in used_types {
        if !registered.contains(t) && is_custom_edge(t) {
            rejected.push(t.clone());
        }
    }
    rejected
}

fn is_custom_edge(s: &str) -> bool {
    matches!(parse_edge_type(s), Ok(EdgeType::Custom(_)))
}
