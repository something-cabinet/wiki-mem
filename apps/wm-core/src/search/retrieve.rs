// Context retrieval from wiki graph with token budget and BM25 fallback.

//! Context retrieval — BFS graph traversal with token budget to build
//! context packs for AI agent consumption.

use petgraph::visit::EdgeRef;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use wm_search::{Bm25Index, Field, IndexedDoc};
use crate::engine::{EdgeType, WikiPageMeta};

/// Retrieve a context pack from the wiki graph with token budget
pub fn retrieve_context(
    graph: &petgraph::stable_graph::StableGraph<WikiPageMeta, EdgeType>,
    id_index: &HashMap<String, petgraph::stable_graph::NodeIndex>,
    query: &str,
    budget: usize,
    bm25_index: Option<&Bm25Index>,
) -> Vec<(String, f64, String)> {
    let budget = budget.clamp(256, 131072);
    let mut results: Vec<(String, f64, String)> = Vec::new(); // (id, score, content_slice)
    let mut tokens_used = 0usize;
    let mut visited: HashSet<String> = HashSet::new();

    // Find the match node
    let match_node = id_index
        .get(query)
        .copied()
        .or_else(|| {
            let results = match bm25_index {
                Some(idx) => idx.search(query, 1),
                None => {
                    // Rebuild BM25 from graph nodes
                    let docs: Vec<IndexedDoc> = graph
                        .node_indices()
                        .map(|idx| {
                            let meta = &graph[idx];
                            IndexedDoc {
                                id: meta.id.clone(),
                                fields: vec![
                                    Field::new("title", &meta.title, 4.0),
                                    Field::new("tags", &meta.tags.join(" "), 2.2),
                                ],
                            }
                        })
                        .collect();
                    Bm25Index::build(docs).search(query, 1)
                }
            };
            results.first().and_then(|r| id_index.get(&r.id)).copied()
        });

    let match_node = match match_node {
        Some(n) => n,
        None => return results,
    };

    let meta = &graph[match_node];
    visited.insert(meta.id.clone());

    // Add match node content with tiered truncation per token budget
    let match_text_full = format!(
        "[MATCH: {}]\nTitle: {}\n{}",
        meta.id,
        meta.title,
        meta.sources.join(", ")
    );
    let match_text_mid = format!("[MATCH: {}]\nTitle: {}", meta.id, meta.title);
    let match_text_min = format!("[MATCH: {}]", meta.id);

    let (match_text, tokens) = {
        let full_tokens = match_text_full.len() / 4;
        if tokens_used + full_tokens <= budget {
            (match_text_full, full_tokens)
        } else {
            let mid_tokens = match_text_mid.len() / 4;
            if tokens_used + mid_tokens <= budget {
                (match_text_mid, mid_tokens)
            } else {
                let min_tokens = match_text_min.len() / 4;
                if tokens_used + min_tokens <= budget {
                    (match_text_min, min_tokens)
                } else {
                    // Can't fit even tier 3 within budget — skip match node entirely
                    (String::new(), 0)
                }
            }
        }
    };
    if tokens > 0 {
        results.push((meta.id.clone(), 999.0, match_text));
        tokens_used += tokens;
    }

    // BFS: collect neighbors with edge-weighted scores
    #[derive(Clone)]
    struct ScoredNeighbor {
        node_idx: petgraph::stable_graph::NodeIndex,
        score: f64,
        edge_type: EdgeType,
    }

    impl PartialEq for ScoredNeighbor {
        fn eq(&self, other: &Self) -> bool {
            self.score == other.score
        }
    }
    impl Eq for ScoredNeighbor {}
    impl PartialOrd for ScoredNeighbor {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for ScoredNeighbor {
        fn cmp(&self, other: &Self) -> Ordering {
            self.score
                .partial_cmp(&other.score)
                .unwrap_or(Ordering::Equal)
        }
    }

    let mut heap: BinaryHeap<ScoredNeighbor> = BinaryHeap::new();

    for edge in graph.edges(match_node) {
        let target = edge.target();
        let id = &graph[target].id;
        if visited.contains(id) {
            continue;
        }
        visited.insert(id.clone());

        let q_lower = query.to_lowercase();
        let title = &graph[target].title.to_lowercase();
        let relevance = if title == &q_lower {
            8.0
        } else if title.contains(&q_lower) {
            4.0
        } else {
            0.0
        };
        let score = edge.weight().priority() as f64 * (1.0 + relevance);
        heap.push(ScoredNeighbor {
            node_idx: target,
            score,
            edge_type: edge.weight().clone(),
        });
    }

    // Process neighbors in priority order, applying structural truncation
    while let Some(sn) = heap.pop() {
        if tokens_used >= budget {
            break;
        }

        let meta = &graph[sn.node_idx];
        let edge_name = format!("{:?}", sn.edge_type).to_lowercase();

        // Tier 1: full content (high relevance)
        if sn.score > 5.0 {
            let text = format!("[{}: {}]\nTitle: {}", edge_name, meta.id, meta.title);
            let tokens = text.len() / 4;
            if tokens_used + tokens <= budget {
                results.push((meta.id.clone(), sn.score, text));
                tokens_used += tokens;
            }
        }
        // Tier 2: frontmatter + headers (medium relevance)
        else if sn.score > 2.0 {
            let text = format!("[{}: {}]", edge_name, meta.id);
            let tokens = text.len() / 4;
            if tokens_used + tokens <= budget {
                results.push((meta.id.clone(), sn.score, text));
                tokens_used += tokens;
            }
        }
        // Tier 3: title + edge type (low relevance)
        else {
            let text = format!("  {} --[{}]--> {}", meta.id, edge_name, meta.title);
            let tokens = text.len() / 4;
            if tokens_used + tokens <= budget {
                results.push((meta.id.clone(), sn.score, text));
                tokens_used += tokens;
            }
        }
    }

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    results
}
