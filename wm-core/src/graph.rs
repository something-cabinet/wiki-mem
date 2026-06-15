use arc_swap::ArcSwap;
use petgraph::stable_graph::StableGraph;
use petgraph::algo::is_cyclic_directed;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};

use crate::engine::{EdgeType, GraphSnapshot, WikiPageMeta};
use crate::parser::{parse_edge_type, parse_wiki_page};

/// Scan a wiki directory and build a StableGraph + id_index
pub fn build_graph_from_wiki(wiki_dir: &Path) -> (StableGraph<WikiPageMeta, EdgeType>, HashMap<String, petgraph::stable_graph::NodeIndex>) {
    let mut graph = StableGraph::<WikiPageMeta, EdgeType>::new();
    let mut id_index = HashMap::new();

    // Find all .md files
    let entries = walkdir::WalkDir::new(wiki_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false));

    for entry in entries {
        let path = entry.path();
        let file_name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();

        // Skip index.md and log.md
        if file_name == "index.md" || file_name == "log.md" {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to read {}: {}", path.display(), e);
                continue;
            }
        };

        if content.trim().is_empty() {
            continue;
        }

        let meta = parse_wiki_page(path, &content);
        let node_idx = graph.add_node(meta.clone());
        id_index.insert(meta.id.clone(), node_idx);
    }

    // Second pass: add edges from relates_to frontmatter
    for entry in walkdir::WalkDir::new(wiki_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
    {
        let path = entry.path();
        let file_name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
        if file_name == "index.md" || file_name == "log.md" {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Extract frontmatter to get relates_to
        let (fm, _) = crate::parser::extract_frontmatter(&content);
        if let Some(fm) = fm {
            if let Some(node_idx) = id_index.get(&parse_wiki_page(path, &content).id) {
                for rel in &fm.relates_to {
                    if let Ok(edge_type) = parse_edge_type(&rel.edge_type) {
                        if let Some(target_idx) = id_index.get(&rel.target) {
                            graph.add_edge(*node_idx, *target_idx, edge_type);
                        }
                    }
                }
            }
        }
    }

    // Cycle detection (diagnostic only, never mutate)
    if is_cyclic_directed(&graph) {
        warn!("Cycle detected in wiki graph. BFS will use visited tracking to prevent infinite loops.");
    } else {
        info!("Graph is acyclic — safe for topological operations.");
    }

    (graph, id_index)
}

/// Rebuild the graph snapshot atomically via ArcSwap
pub fn rebuild_snapshot(
    graph_swap: &ArcSwap<GraphSnapshot>,
    wiki_dir: &Path,
) -> usize {
    let (graph, id_index) = build_graph_from_wiki(wiki_dir);
    let node_count = graph.node_count();
    let snapshot = Arc::new((graph, id_index));
    graph_swap.store(snapshot);
    info!("Graph rebuilt: {} nodes, replaced atomically.", node_count);
    node_count
}

/// Validate custom edge types against a registered set
pub fn validate_custom_edge_types(
    registered: &[String],
    used_types: &[String],
) -> Vec<String> {
    let mut rejected = Vec::new();
    for t in used_types {
        if !registered.contains(t) && is_custom_edge(t) {
            rejected.push(t.clone());
        }
    }
    rejected
}

fn is_custom_edge(s: &str) -> bool {
    matches!(
        parse_edge_type(s),
        Ok(EdgeType::Custom(_))
    )
}
