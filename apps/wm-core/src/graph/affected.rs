//! Blast-radius / impact analysis.
//!
//! `affected` returns the transitive breakage set for a node: everything that
//! would break if the node were removed. Traversal follows **incoming**
//! break-sensitive edges, because a stored edge points from the dependent to
//! its dependency (`caller → callee`, `importer → imported`, `X depends_on Y`):
//!
//! - code:   `calls`, `inherits`, `implements`, `imports`
//! - wiki:   `depends_on`, `extends`
//!
//! Each affected node carries the edge path from the start node with the
//! provenance (and file:line for code edges) of every hop.

use petgraph::stable_graph::StableGraph;
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

use crate::engine::{EdgeProvenance, EdgeType, GraphEdge, WikiPageMeta};

/// One hop on the path from the queried node to an affected node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AffectedHop {
    pub edge_type: String,
    /// Node id of the hop's source endpoint (the dependent).
    pub from: String,
    /// Node id of the hop's target endpoint (the dependency).
    pub to: String,
    /// 1-based source line for code edges; `None` for wiki edges.
    pub line: Option<usize>,
    pub provenance: EdgeProvenance,
}

/// A transitively affected node with the full edge path from the start node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AffectedNode {
    pub node_id: String,
    pub title: String,
    /// Hops from the queried node to this node (shortest path, BFS order).
    pub hops: Vec<AffectedHop>,
}

impl AffectedNode {
    pub fn depth(&self) -> usize {
        self.hops.len()
    }
}

/// Break-sensitive wiki edge types: a page that `depends_on` or `extends`
/// another page breaks when the dependency is removed.
fn is_wiki_break_sensitive(edge_type: &EdgeType) -> bool {
    matches!(edge_type, EdgeType::DependsOn | EdgeType::Extends)
}

/// Compute the transitive breakage set of a wiki page node.
///
/// Traverses INCOMING `depends_on`/`extends` edges: page P has an edge
/// `P depends_on D`, so removing D breaks P.
pub fn affected_wiki_nodes(
    graph: &StableGraph<WikiPageMeta, GraphEdge>,
    start: petgraph::stable_graph::NodeIndex,
    max_depth: usize,
) -> Vec<AffectedNode> {
    let mut out: Vec<AffectedNode> = Vec::new();
    let mut seen: HashSet<petgraph::stable_graph::NodeIndex> = HashSet::new();
    let mut queue: VecDeque<(petgraph::stable_graph::NodeIndex, Vec<AffectedHop>)> =
        VecDeque::new();
    seen.insert(start);
    queue.push_back((start, Vec::new()));

    while let Some((current, hops)) = queue.pop_front() {
        if hops.len() >= max_depth {
            continue;
        }
        for edge in graph.edges_directed(current, petgraph::Direction::Incoming) {
            let source = edge.source();
            let edge_type = &edge.weight().edge_type;
            if !is_wiki_break_sensitive(edge_type) {
                continue;
            }
            let mut next_hops = hops.clone();
            next_hops.push(AffectedHop {
                edge_type: edge_type.as_yaml_str().to_string(),
                from: graph[source].id.clone(),
                to: graph[current].id.clone(),
                line: None,
                provenance: edge.weight().provenance,
            });
            if seen.insert(source) {
                out.push(AffectedNode {
                    node_id: graph[source].id.clone(),
                    title: graph[source].title.clone(),
                    hops: next_hops.clone(),
                });
                queue.push_back((source, next_hops));
            }
        }
    }

    out.sort_by(|a, b| a.node_id.cmp(&b.node_id));
    out
}

#[cfg(feature = "code-intel")]
pub mod code {
    use super::*;
    use std::collections::{HashSet, VecDeque};

    use wm_code_intel::services::graph_resolver::{CodeEdgeGraph, CodeNodeRef, ResolvedCodeEdge};

    /// Compute the transitive breakage set of a code node.
    ///
    /// Traverses INCOMING break-sensitive edges:
    /// - symbol node: incoming `calls`/`inherits` (callers / implementers);
    /// - file node: incoming `imports` (importers) + incoming `calls`/`inherits`
    ///   targeting symbols defined in the file.
    pub fn affected_code_nodes(
        graph: &CodeEdgeGraph,
        start: &CodeNodeRef,
        max_depth: usize,
    ) -> Vec<AffectedNode> {
        let mut out: Vec<AffectedNode> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<(CodeNodeRef, Vec<AffectedHop>)> = VecDeque::new();
        seen.insert(start.node_id());
        queue.push_back((start.clone(), Vec::new()));

        while let Some((current, hops)) = queue.pop_front() {
            if hops.len() >= max_depth {
                continue;
            }
            let incoming: Vec<&ResolvedCodeEdge> = match &current {
                CodeNodeRef::Symbol { file, symbol } => graph.incoming_to_symbol(file, symbol),
                CodeNodeRef::File(file) => graph.incoming_to_file(file),
                CodeNodeRef::SymbolName(name) => graph.incoming_to_symbol_name(name),
            };
            for edge in incoming {
                let from_id = edge.source_node_id();
                let mut next_hops = hops.clone();
                next_hops.push(AffectedHop {
                    edge_type: edge.edge_type.clone(),
                    from: from_id.clone(),
                    to: edge.target_node_id(),
                    line: Some(edge.line),
                    provenance: edge.provenance,
                });
                if seen.insert(from_id.clone()) {
                    let node = CodeNodeRef::parse(&from_id, graph);
                    out.push(AffectedNode {
                        node_id: from_id,
                        title: node.title(),
                        hops: next_hops.clone(),
                    });
                    queue.push_back((node, next_hops));
                }
            }
        }

        out.sort_by(|a, b| a.node_id.cmp(&b.node_id));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::build_graph_from_wiki;

    fn write_page(wiki_dir: &std::path::Path, rel: &str, content: &str) {
        let full = wiki_dir.join(rel);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(&full, content).unwrap();
    }

    #[test]
    fn ac62_wiki_depends_on_extends_in_affected_set() {
        let tmp = tempfile::TempDir::new().unwrap();
        let wiki_dir = tmp.path().join(".wm").join("wiki");
        std::fs::create_dir_all(wiki_dir.join("core")).unwrap();
        std::fs::create_dir_all(wiki_dir.join("concepts")).unwrap();

        write_page(
            &wiki_dir,
            "core/db.md",
            "---\ntitle: DB\ntype: core\n---\n\nDB.\n",
        );
        write_page(
            &wiki_dir,
            "core/repo.md",
            r#"---
title: Repo
type: core
relates_to:
  - type: depends_on
    target: wiki:core:db
---
Repo.
"#,
        );
        write_page(
            &wiki_dir,
            "concepts/service.md",
            r#"---
title: Service
type: concept
relates_to:
  - type: depends_on
    target: wiki:core:repo
  - type: extends
    target: wiki:core:repo
---
Service.
"#,
        );
        write_page(
            &wiki_dir,
            "concepts/unrelated.md",
            "---\ntitle: Unrelated\ntype: concept\n---\n\nUnrelated.\n",
        );

        let (graph, id_index) = build_graph_from_wiki(wiki_dir.as_path(), &[]);
        let db_idx = *id_index.get("wiki:core:db").expect("db node");
        let affected = affected_wiki_nodes(&graph, db_idx, 10);

        let ids: Vec<&str> = affected.iter().map(|a| a.node_id.as_str()).collect();
        assert!(
            ids.contains(&"wiki:core:repo"),
            "repo depends_on db must be affected: {:?}",
            ids
        );
        assert!(
            ids.contains(&"wiki:concepts:service"),
            "service depends_on repo (→ db) must be transitively affected: {:?}",
            ids
        );
        assert!(
            !ids.contains(&"wiki:concepts:unrelated"),
            "unrelated page must not be affected"
        );

        let service = affected
            .iter()
            .find(|a| a.node_id == "wiki:concepts:service")
            .unwrap();
        assert_eq!(
            service.hops.len(),
            2,
            "service is 2 hops from db (via repo)"
        );
        let hop_types: Vec<&str> = service.hops.iter().map(|h| h.edge_type.as_str()).collect();
        assert!(
            hop_types
                .iter()
                .all(|t| *t == "depends_on" || *t == "extends"),
            "hop types are break-sensitive wiki edges: {:?}",
            hop_types
        );
    }

    #[test]
    fn affected_ignores_non_break_sensitive_wiki_edges() {
        let tmp = tempfile::TempDir::new().unwrap();
        let wiki_dir = tmp.path().join(".wm").join("wiki");
        std::fs::create_dir_all(wiki_dir.join("core")).unwrap();
        std::fs::create_dir_all(wiki_dir.join("concepts")).unwrap();

        write_page(
            &wiki_dir,
            "core/target.md",
            "---\ntitle: Target\ntype: core\n---\n\nTarget.\n",
        );
        write_page(
            &wiki_dir,
            "concepts/source.md",
            r#"---
title: Source
type: concept
relates_to:
  - type: references
    target: wiki:core:target
---
Source.
"#,
        );

        let (graph, id_index) = build_graph_from_wiki(wiki_dir.as_path(), &[]);
        let target_idx = *id_index.get("wiki:core:target").expect("target node");
        let affected = affected_wiki_nodes(&graph, target_idx, 10);
        assert!(
            affected.is_empty(),
            "references edges must not appear in the affected set"
        );
    }

    #[test]
    fn wiki_affected_deterministic() {
        let tmp = tempfile::TempDir::new().unwrap();
        let wiki_dir = tmp.path().join(".wm").join("wiki");
        std::fs::create_dir_all(wiki_dir.join("core")).unwrap();

        write_page(
            &wiki_dir,
            "core/db.md",
            "---\ntitle: DB\ntype: core\n---\n\nDB.\n",
        );
        for name in ["aa", "bb", "cc"] {
            write_page(
                &wiki_dir,
                &format!("core/{}.md", name),
                &format!(
                    "---\ntitle: {}\ntype: core\nrelates_to:\n  - type: depends_on\n    target: wiki:core:db\n---\n\n{}\n",
                    name, name
                ),
            );
        }

        let (graph, id_index) = build_graph_from_wiki(wiki_dir.as_path(), &[]);
        let db_idx = *id_index.get("wiki:core:db").expect("db node");
        let a = affected_wiki_nodes(&graph, db_idx, 10);
        let b = affected_wiki_nodes(&graph, db_idx, 10);
        assert_eq!(a, b, "affected set is deterministic");
        assert_eq!(a.len(), 3);
        let ids: Vec<&str> = a.iter().map(|n| n.node_id.as_str()).collect();
        assert!(ids.windows(2).all(|w| w[0] < w[1]), "sorted by node id");
    }

    #[cfg(feature = "code-intel")]
    mod code_tests {
        use super::*;
        use crate::graph::affected::code::affected_code_nodes;
        use wm_code_intel::models::code_edge_model::CodeEdge;
        use wm_code_intel::models::symbol_model::CodeIntelSymbol;
        use wm_code_intel::services::graph_resolver::{
            resolve_code_edges, CodeEdgeGraph, CodeIndexSnapshot, CodeNodeRef,
        };

        fn symbol(file: &str, name: &str, kind: &str, line: usize) -> CodeIntelSymbol {
            CodeIntelSymbol {
                file: file.to_string(),
                name: name.to_string(),
                kind: kind.to_string(),
                line,
                column: 0,
                snippet: String::new(),
                language: "rust".to_string(),
            }
        }

        fn raw_edge(
            edge_type: &str,
            source_file: &str,
            source_symbol: Option<&str>,
            target_symbol: Option<&str>,
            line: usize,
            provenance: EdgeProvenance,
        ) -> CodeEdge {
            CodeEdge {
                edge_type: edge_type.to_string(),
                source_file: source_file.to_string(),
                source_symbol: source_symbol.map(|s| s.to_string()),
                target_file: String::new(),
                target_symbol: target_symbol.map(|s| s.to_string()),
                receiver: None,
                line,
                provenance,
            }
        }

        fn build_graph(
            edges: Vec<CodeEdge>,
            symbols: Vec<CodeIntelSymbol>,
        ) -> (
            CodeEdgeGraph,
            Vec<wm_code_intel::services::graph_resolver::ResolvedCodeEdge>,
        ) {
            let mut files: std::collections::HashSet<String> =
                symbols.iter().map(|s| s.file.clone()).collect();
            for e in &edges {
                files.insert(e.source_file.clone());
            }
            let snapshot = CodeIndexSnapshot {
                symbols,
                raw_edges: edges,
                files,
                ts_context: None,
            };
            let resolved = resolve_code_edges(&snapshot);
            (CodeEdgeGraph::build(resolved.clone()), resolved)
        }

        #[test]
        fn ac61_removing_function_lists_transitive_callers() {
            let (graph, _) = build_graph(
                vec![
                    raw_edge(
                        "calls",
                        "src/engine.rs",
                        Some("run"),
                        Some("step"),
                        3,
                        EdgeProvenance::Explicit,
                    ),
                    raw_edge(
                        "calls",
                        "src/main.rs",
                        Some("main"),
                        Some("run"),
                        7,
                        EdgeProvenance::Explicit,
                    ),
                ],
                vec![
                    symbol("src/step.rs", "step", "function", 1),
                    symbol("src/engine.rs", "run", "function", 1),
                    symbol("src/main.rs", "main", "function", 1),
                ],
            );

            let start = CodeNodeRef::parse("src/step.rs#step", &graph);
            let affected = affected_code_nodes(&graph, &start, 10);

            let ids: Vec<&str> = affected.iter().map(|a| a.node_id.as_str()).collect();
            assert!(
                ids.contains(&"src/engine.rs#run"),
                "run calls step — affected: {:?}",
                ids
            );
            assert!(
                ids.contains(&"src/main.rs#main"),
                "main calls run — transitively affected: {:?}",
                ids
            );

            let main_node = affected
                .iter()
                .find(|a| a.node_id == "src/main.rs#main")
                .unwrap();
            assert_eq!(main_node.hops.len(), 2, "main is 2 hops from step");
            assert_eq!(main_node.hops[0].edge_type, "calls");
            assert_eq!(main_node.hops[0].from, "src/engine.rs#run");
            assert_eq!(main_node.hops[0].to, "src/step.rs#step");
            assert_eq!(main_node.hops[0].line, Some(3));
            assert_eq!(main_node.hops[0].provenance, EdgeProvenance::Explicit);
            assert_eq!(main_node.hops[1].from, "src/main.rs#main");
            assert_eq!(main_node.hops[1].to, "src/engine.rs#run");
            assert_eq!(main_node.hops[1].line, Some(7));
        }

        #[test]
        fn ac61_file_node_includes_importers() {
            let (graph, _) = build_graph(
                vec![raw_edge(
                    "imports",
                    "src/main.rs",
                    None,
                    Some("crate::lib"),
                    2,
                    EdgeProvenance::Explicit,
                )],
                vec![symbol("src/lib.rs", "helper", "function", 1)],
            );

            let start = CodeNodeRef::parse("src/lib.rs", &graph);
            let affected = affected_code_nodes(&graph, &start, 10);
            let ids: Vec<&str> = affected.iter().map(|a| a.node_id.as_str()).collect();
            assert_eq!(ids, vec!["src/main.rs"], "importer is affected");
            assert_eq!(affected[0].hops[0].edge_type, "imports");
        }

        #[test]
        fn parse_code_node_ref_forms() {
            let (graph, _) = build_graph(
                vec![raw_edge(
                    "imports",
                    "src/main.rs",
                    None,
                    Some("crate::lib"),
                    2,
                    EdgeProvenance::Explicit,
                )],
                vec![symbol("src/lib.rs", "helper", "function", 1)],
            );
            match CodeNodeRef::parse("src/lib.rs#helper", &graph) {
                CodeNodeRef::Symbol { file, symbol } => {
                    assert_eq!(file, "src/lib.rs");
                    assert_eq!(symbol, "helper");
                }
                other => panic!("expected symbol ref, got {:?}", other),
            }
            assert!(matches!(
                CodeNodeRef::parse("src/lib.rs", &graph),
                CodeNodeRef::File(_)
            ));
            assert!(matches!(
                CodeNodeRef::parse("nope.rs", &graph),
                CodeNodeRef::SymbolName(_)
            ));
        }
    }
}
