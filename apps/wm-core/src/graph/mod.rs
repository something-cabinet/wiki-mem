use petgraph::stable_graph::StableGraph;
use std::collections::HashMap;
use std::path::Path;
use tracing::info;
use wm_constants::*;

use crate::engine::{EdgeProvenance, EdgeType, GraphEdge, GraphSnapshot, WikiPageMeta};

pub mod affected;
#[cfg(feature = "code-intel")]
pub mod code_edges;
pub mod export;
pub mod index_gen;
pub mod lint;
pub mod path;
pub mod sections;

pub use affected::*;
#[cfg(feature = "code-intel")]
pub use code_edges::*;
pub use export::*;
pub use index_gen::*;
pub use lint::*;
pub use path::*;
pub use sections::*;

/// Serializes full-graph rebuilds. Concurrent `handle_file_change` calls (e.g.
/// parallel page creates) each trigger a `rebuild_graph_snapshot`; without a
/// lock, a rebuild that starts early but finishes last can win the final
/// `ArcSwap::store` with a stale scan that misses concurrently-written pages.
static REBUILD_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub fn rebuild_graph_snapshot(
    graph_swap: &arc_swap::ArcSwap<GraphSnapshot>,
    wiki_dir: &Path,
    custom_types: &[String],
) -> usize {
    // Serialize so the last rebuild to store reflects the latest dir state.
    let _guard = REBUILD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (graph, id_index) = build_graph_from_wiki(wiki_dir, custom_types);
    let node_count = graph.node_count();
    let snapshot = std::sync::Arc::new((graph, id_index));
    graph_swap.store(snapshot);
    info!("Graph rebuilt: {} nodes, replaced atomically.", node_count);
    node_count
}

struct ParsedPage {
    meta: WikiPageMeta,
    edges: Vec<(GraphEdge, String)>,
    custom_types: Vec<String>,
    body_extracted_targets: Vec<String>,
}

pub fn build_graph_from_wiki(
    wiki_dir: &Path,
    registered_custom_types: &[String],
) -> (
    StableGraph<WikiPageMeta, GraphEdge>,
    HashMap<String, petgraph::stable_graph::NodeIndex>,
) {
    use petgraph::algo::is_cyclic_directed;
    use rayon::prelude::*;
    use tracing::warn;

    let mut graph = StableGraph::<WikiPageMeta, GraphEdge>::new();
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
            let rel_path = Path::new(WM_DIR).join(WIKI_DIR).join(wiki_rel);
            let content = std::fs::read_to_string(path).ok()?;
            if content.trim().is_empty() {
                return None;
            }
            let meta = parse_wiki_page(&rel_path, &content);
            let mut edges: Vec<(GraphEdge, String)> = Vec::new();
            let mut custom_types: Vec<String> = Vec::new();
            for (edge_type, target) in &meta.relates_to {
                let edge_type_str = match edge_type {
                    EdgeType::Custom(name) => name.to_lowercase(),
                    _ => format!("{:?}", edge_type).to_lowercase(),
                };
                edges.push((
                    GraphEdge::new(edge_type.clone(), EdgeProvenance::Explicit),
                    target.clone(),
                ));
                if is_custom_edge(&edge_type_str) && !custom_types.contains(&edge_type_str) {
                    custom_types.push(edge_type_str);
                }
            }

            let (_, body) = crate::parser::extract_frontmatter(&content);
            let body_refs = crate::reference::extract_references(body);
            let mut body_extracted_targets: Vec<String> = Vec::new();
            for r in body_refs {
                let target = format!("wiki:{}:{}", r.ref_type, r.target);
                let already_from_fm = edges
                    .iter()
                    .any(|(ge, t)| ge.edge_type == EdgeType::References && *t == target);
                if !already_from_fm {
                    edges.push((
                        GraphEdge::new(EdgeType::References, EdgeProvenance::Explicit),
                        target.clone(),
                    ));
                    body_extracted_targets.push(target);
                }
            }

            Some(ParsedPage {
                meta,
                edges,
                custom_types,
                body_extracted_targets,
            })
        })
        .collect();

    let mut pending_edges: Vec<(String, GraphEdge, String)> = Vec::new();
    let mut used_custom_types: Vec<String> = Vec::new();

    for page in &parsed {
        let node_idx = graph.add_node(page.meta.clone());
        id_index.insert(page.meta.id.clone(), node_idx);
        for (edge, target) in &page.edges {
            pending_edges.push((page.meta.id.clone(), edge.clone(), target.clone()));
        }
        for ct in &page.custom_types {
            if !used_custom_types.contains(ct) {
                used_custom_types.push(ct.clone());
            }
        }
    }

    for page in &parsed {
        for target in &page.body_extracted_targets {
            let already_exists = pending_edges.iter().any(|(src, et, tgt)| {
                src == target && et.edge_type == EdgeType::References && tgt == &page.meta.id
            });
            if !already_exists {
                pending_edges.push((
                    target.clone(),
                    GraphEdge::new(EdgeType::References, EdgeProvenance::Derived),
                    page.meta.id.clone(),
                ));
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
        String,
    )> = std::collections::HashSet::new();
    for (source_id, edge, target) in &pending_edges {
        let edge_type_str = match &edge.edge_type {
            EdgeType::Custom(name) => name.to_lowercase(),
            _ => format!("{:?}", edge.edge_type).to_lowercase(),
        };
        if rejected.contains(&edge_type_str) {
            continue;
        }
        if let Some(&source_idx) = id_index.get(source_id) {
            let normalized_target = target.replace('/', ":");
            let mut provenance = edge.provenance;
            let target_idx = if let Some(&idx) = id_index.get(&normalized_target) {
                Some(idx)
            } else if let Some(&idx) = id_index.get(target) {
                Some(idx)
            } else {
                let candidates = crate::parser::resolve_link_target_candidates(target, &graph);
                if candidates.len() > 1 {
                    provenance = EdgeProvenance::Ambiguous;
                }
                candidates.first().and_then(|id| id_index.get(id).copied())
            };
            if target_idx.is_none() {
                tracing::warn!(
                    "Graph: unresolved relates_to target '{}' from '{}'",
                    target,
                    source_id
                );
            }
            if let Some(target_idx) = target_idx {
                let edge_weight = if provenance == edge.provenance {
                    edge.clone()
                } else {
                    GraphEdge::new(edge.edge_type.clone(), provenance)
                };
                if added_edges.insert((source_idx, target_idx, edge_type_str)) {
                    graph.add_edge(source_idx, target_idx, edge_weight);
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

use crate::engine::{EngineState, SectionDoc};
use crate::search::{Bm25Index, IndexedDoc};
use std::sync::Arc;

fn rebuild_bm25_from_corpus(engine: &EngineState) {
    let corpus = engine.section_corpus.load();
    let docs: Vec<IndexedDoc> = corpus
        .iter()
        .map(crate::search::indexed_doc_from_section)
        .collect();
    engine.bm25_index.store(Arc::new(Bm25Index::build(docs)));
}

pub fn handle_file_change(wiki_dir: &Path, path: &Path, engine: &EngineState) {
    if path.extension().is_none_or(|e| e != "md") {
        return;
    }
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if file_name == "index.md" || file_name == "log.md" {
        return;
    }

    tracing::info!("File change detected: {}", path.display());

    let custom_types = match engine.config.read() {
        Ok(cfg) => cfg.custom_edge_types.clone(),
        Err(_) => {
            tracing::error!("Config lock poisoned in handle_file_change");
            return;
        }
    };

    rebuild_graph_snapshot(&engine.graph, wiki_dir, &custom_types);

    let snapshot = engine.graph.load();
    if let Err(e) = auto_generate_index(wiki_dir, &snapshot.0) {
        tracing::warn!("Failed to regenerate index.md: {}", e);
    }
    drop(snapshot);

    if let Some(sections) = build_sections_from_file(path) {
        let page_id = sections
            .first()
            .map(|s| s.page_id.clone())
            .unwrap_or_default();

        crate::page::services::page_crud_service::update_vectors_for_page(
            engine, &page_id, &sections, false,
        );

        let page_id = page_id.clone();
        engine.section_corpus.rcu(|old| {
            let mut c: Vec<SectionDoc> = (**old).clone();
            c.retain(|s| s.page_id != page_id);
            c.extend(sections.clone());
            Arc::new(c)
        });
    }

    rebuild_bm25_from_corpus(engine);

    engine.notify_file_changed(path);

    engine
        .stale_flag
        .store(false, std::sync::atomic::Ordering::Release);

    tracing::info!("File change handled: {}", path.display());
}

pub fn handle_file_delete(wiki_dir: &Path, path: &Path, engine: &EngineState) {
    if path.extension().is_none_or(|e| e != "md") {
        return;
    }
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if file_name == "index.md" || file_name == "log.md" {
        return;
    }

    tracing::info!("File delete detected: {}", path.display());

    let rel_path = path.strip_prefix(wiki_dir).unwrap_or(path);
    let page_id = crate::parser::path_to_id(&rel_path.to_string_lossy());

    let custom_types = match engine.config.read() {
        Ok(cfg) => cfg.custom_edge_types.clone(),
        Err(_) => {
            tracing::error!("Config lock poisoned in handle_file_delete");
            return;
        }
    };

    rebuild_graph_snapshot(&engine.graph, wiki_dir, &custom_types);

    let snapshot = engine.graph.load();
    if let Err(e) = auto_generate_index(wiki_dir, &snapshot.0) {
        tracing::warn!("Failed to regenerate index.md: {}", e);
    }
    drop(snapshot);

    let pid = page_id.clone();
    engine.section_corpus.rcu(|old| {
        let mut c: Vec<SectionDoc> = (**old).clone();
        c.retain(|s| s.page_id != pid);
        Arc::new(c)
    });

    crate::page::services::page_crud_service::update_vectors_for_page(engine, &page_id, &[], true);

    rebuild_bm25_from_corpus(engine);

    engine
        .stale_flag
        .store(false, std::sync::atomic::Ordering::Release);

    tracing::info!("File delete handled: {}", path.display());
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
    matches!(EdgeType::from_str_flexible(s), EdgeType::Custom(_))
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::visit::EdgeRef;
    use tempfile::TempDir;

    #[test]
    fn test_body_ref_extraction_and_reciprocal_edges() {
        let tmp = TempDir::new().unwrap();
        let wiki_dir = tmp.path().join(".wm").join("wiki");
        std::fs::create_dir_all(wiki_dir.join("concepts")).unwrap();
        std::fs::create_dir_all(wiki_dir.join("patterns")).unwrap();
        std::fs::create_dir_all(wiki_dir.join("tasks")).unwrap();
        std::fs::create_dir_all(wiki_dir.join("testing")).unwrap();

        std::fs::write(
            wiki_dir.join("concepts/bm25-search.md"),
            "See @wiki/patterns/field-weighted-bm25 for scoring details.\n",
        )
        .unwrap();

        std::fs::write(
            wiki_dir.join("patterns/field-weighted-bm25.md"),
            r#"---
type: pattern
---

Used by @wiki/concepts/bm25-search and @wiki/tasks/task-g2gckv-bm25-search-onnx-embeddings
"#,
        )
        .unwrap();

        std::fs::write(
            wiki_dir.join("tasks/task-g2gckv-bm25-search-onnx-embeddings.md"),
            "# Task\n\nDo the thing.\n",
        )
        .unwrap();

        std::fs::write(
            wiki_dir.join("concepts/graph-architecture.md"),
            "# Graph Architecture\n\nDesign notes.\n",
        )
        .unwrap();

        std::fs::write(
            wiki_dir.join("testing/dedup-test.md"),
            r#"---
relates_to:
  - type: references
    target: wiki:concepts:graph-architecture
---

Some text with @wiki/concepts/graph-architecture
"#,
        )
        .unwrap();

        let (graph, id_index) = build_graph_from_wiki(wiki_dir.as_path(), &[]);

        eprintln!("=== Nodes ({}) ===", graph.node_count());
        for (id, idx) in &id_index {
            eprintln!("  {} -> {:?}", id, idx);
        }
        eprintln!("=== Edges ({}) ===", graph.edge_count());
        for edge_idx in graph.edge_indices() {
            let (src, dst) = graph.edge_endpoints(edge_idx).unwrap();
            let w = graph.edge_weight(edge_idx).unwrap();
            let src_id = &graph[src].id;
            let dst_id = &graph[dst].id;
            eprintln!("  {:?} {} -> {}", w, src_id, dst_id);
        }

        assert!(
            graph.edge_count() > 0,
            "should have at least one edge from body @wiki/ extraction"
        );

        let edge_exists = |from: &str, to: &str| -> bool {
            match (id_index.get(from), id_index.get(to)) {
                (Some(&f), Some(&t)) => graph
                    .edges_connecting(f, t)
                    .any(|e| e.weight().edge_type == EdgeType::References),
                _ => false,
            }
        };

        assert!(
            edge_exists(
                "wiki:concepts:bm25-search",
                "wiki:patterns:field-weighted-bm25"
            ),
            "missing references edge from concepts:bm25-search → patterns:field-weighted-bm25"
        );

        assert!(
            edge_exists(
                "wiki:patterns:field-weighted-bm25",
                "wiki:concepts:bm25-search"
            ),
            "missing reciprocal references edge from patterns:field-weighted-bm25 → concepts:bm25-search"
        );

        assert!(
            edge_exists(
                "wiki:patterns:field-weighted-bm25",
                "wiki:tasks:task-g2gckv-bm25-search-onnx-embeddings"
            ),
            "missing references edge from patterns:field-weighted-bm25 → task"
        );

        let dedup_id = "wiki:testing:dedup-test";
        let arch_id = "wiki:concepts:graph-architecture";
        let (dedup_idx, arch_idx) = match (id_index.get(dedup_id), id_index.get(arch_id)) {
            (Some(&a), Some(&b)) => (a, b),
            _ => panic!("dedup-test or graph-architecture node not found in graph"),
        };

        let ref_edge_count = graph
            .edges_connecting(dedup_idx, arch_idx)
            .filter(|e| e.weight().edge_type == EdgeType::References)
            .count();

        assert_eq!(
            ref_edge_count, 1,
            "expected exactly 1 references edge (frontmatter takes precedence, no duplicate), got {ref_edge_count}"
        );
    }

    #[test]
    fn test_edge_provenance_fixture() {
        let tmp = TempDir::new().unwrap();
        let wiki_dir = tmp.path().join(".wm").join("wiki");
        std::fs::create_dir_all(wiki_dir.join("concepts")).unwrap();
        std::fs::create_dir_all(wiki_dir.join("patterns")).unwrap();

        std::fs::write(
            wiki_dir.join("concepts/author-source.md"),
            r#"---
relates_to:
  - type: references
    target: wiki:patterns:author-target
---

Authored frontmatter edge.
"#,
        )
        .unwrap();

        std::fs::write(
            wiki_dir.join("patterns/author-target.md"),
            "See @wiki/concepts/recip-source for details.\n",
        )
        .unwrap();

        std::fs::write(
            wiki_dir.join("concepts/recip-source.md"),
            "# Recip Source\n\nPlain page.\n",
        )
        .unwrap();

        std::fs::write(
            wiki_dir.join("concepts/ambig-a.md"),
            "# Ambig A\n\nFirst ambiguous candidate.\n",
        )
        .unwrap();

        std::fs::write(
            wiki_dir.join("patterns/ambig-a.md"),
            "# Ambig A\n\nSecond ambiguous candidate.\n",
        )
        .unwrap();

        std::fs::write(
            wiki_dir.join("concepts/ambig-source.md"),
            r#"---
relates_to:
  - type: references
    target: ambig-a
---

Intentionally ambiguous short target.
"#,
        )
        .unwrap();

        let (graph, id_index) = build_graph_from_wiki(wiki_dir.as_path(), &[]);

        let find_edge = |from: &str, to: &str| -> Vec<EdgeProvenance> {
            match (id_index.get(from), id_index.get(to)) {
                (Some(&f), Some(&t)) => graph
                    .edges_connecting(f, t)
                    .map(|e| e.weight().provenance)
                    .collect(),
                _ => Vec::new(),
            }
        };

        let authored_from_fm =
            find_edge("wiki:concepts:author-source", "wiki:patterns:author-target");
        assert_eq!(
            authored_from_fm,
            vec![EdgeProvenance::Explicit],
            "frontmatter relates_to edge must be explicit"
        );

        let authored_body = find_edge("wiki:patterns:author-target", "wiki:concepts:recip-source");
        assert_eq!(
            authored_body,
            vec![EdgeProvenance::Explicit],
            "authored @wiki body ref edge must be explicit"
        );

        let reciprocal = find_edge("wiki:concepts:recip-source", "wiki:patterns:author-target");
        assert_eq!(
            reciprocal,
            vec![EdgeProvenance::Derived],
            "auto-created reciprocal backlink edge must be derived"
        );

        let ambig_source = "wiki:concepts:ambig-source";
        let ambig_idx = id_index.get(ambig_source).copied().unwrap();
        let ambiguous_edges: Vec<_> = graph
            .edges(ambig_idx)
            .map(|e| (graph[e.target()].id.clone(), e.weight().provenance))
            .collect();
        assert_eq!(
            ambiguous_edges.len(),
            1,
            "ambig-source must have exactly 1 edge"
        );
        let (ambig_target, ambig_prov) = &ambiguous_edges[0];
        assert_eq!(
            *ambig_prov,
            EdgeProvenance::Ambiguous,
            "multi-candidate resolution edge must be ambiguous"
        );
        assert!(
            ambig_target == "wiki:concepts:ambig-a" || ambig_target == "wiki:patterns:ambig-a",
            "ambiguous edge must target one of the candidate pages, got {ambig_target}"
        );

        // FR-1.2: rebuild is deterministic — same provenance on the same edges.
        let (graph2, _) = build_graph_from_wiki(wiki_dir.as_path(), &[]);
        let re_find = |from: &str, to: &str| -> Vec<EdgeProvenance> {
            match (id_index.get(from), id_index.get(to)) {
                (Some(&f), Some(&t)) => graph2
                    .edges_connecting(f, t)
                    .map(|e| e.weight().provenance)
                    .collect(),
                _ => Vec::new(),
            }
        };
        assert_eq!(
            re_find("wiki:concepts:author-source", "wiki:patterns:author-target"),
            vec![EdgeProvenance::Explicit]
        );
        assert_eq!(
            re_find("wiki:concepts:recip-source", "wiki:patterns:author-target"),
            vec![EdgeProvenance::Derived]
        );
        let ambig2 = id_index.get(ambig_source).copied().unwrap();
        assert_eq!(
            graph2.edges(ambig2).next().map(|e| e.weight().provenance),
            Some(EdgeProvenance::Ambiguous)
        );
    }

    #[test]
    fn test_all_explicit_edges_are_neutral() {
        let tmp = TempDir::new().unwrap();
        let wiki_dir = tmp.path().join(".wm").join("wiki");
        std::fs::create_dir_all(wiki_dir.join("concepts")).unwrap();
        std::fs::create_dir_all(wiki_dir.join("patterns")).unwrap();

        std::fs::write(
            wiki_dir.join("concepts/a.md"),
            r#"---
relates_to:
  - type: references
    target: wiki:patterns:b
---

A.
"#,
        )
        .unwrap();

        std::fs::write(
            wiki_dir.join("patterns/b.md"),
            r#"---
relates_to:
  - type: references
    target: wiki:concepts:a
---

B.
"#,
        )
        .unwrap();

        let (graph, _id_index) = build_graph_from_wiki(wiki_dir.as_path(), &[]);

        assert_eq!(EdgeProvenance::Explicit.factor(), 1.0);
        for edge_idx in graph.edge_indices() {
            assert_eq!(
                graph[edge_idx].provenance,
                EdgeProvenance::Explicit,
                "frontmatter-authored edges must stay explicit (neutral weight)"
            );
        }
    }
}
