use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use arc_swap::{ArcSwap, Guard};
use petgraph::stable_graph::StableGraph;

use crate::engine::GraphSnapshot;

pub struct GraphState {
    graph: ArcSwap<GraphSnapshot>,
}

impl Default for GraphState {
    fn default() -> Self {
        Self {
            graph: ArcSwap::new(Arc::new((StableGraph::new(), HashMap::new()))),
        }
    }
}

impl GraphState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> Guard<Arc<GraphSnapshot>> {
        self.graph.load()
    }

    pub(crate) fn replace(&self, snapshot: Arc<GraphSnapshot>) {
        self.graph.store(snapshot);
    }

    pub(crate) fn update<F>(&self, update: F)
    where
        F: Fn(&GraphSnapshot) -> GraphSnapshot,
    {
        self.graph.rcu(|current| Arc::new(update(current)));
    }

    pub fn rebuild(&self, wiki_dir: &Path, custom_types: &[String]) -> usize {
        crate::graph::rebuild_graph_snapshot(self, wiki_dir, custom_types)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{GraphEdge, WikiPageMeta};

    const GRAPH_FIELDS: [&str; 1] = ["graph"];

    fn sample_meta(id: &str) -> WikiPageMeta {
        WikiPageMeta {
            id: id.to_string(),
            title: format!("Page {id}"),
            ..WikiPageMeta::default()
        }
    }

    fn sample_snapshot(id: &str) -> GraphSnapshot {
        let mut graph = StableGraph::<WikiPageMeta, GraphEdge>::new();
        let node = graph.add_node(sample_meta(id));
        let mut index = HashMap::new();
        index.insert(id.to_string(), node);
        (graph, index)
    }

    fn graph_struct_body() -> &'static str {
        const SOURCE: &str = include_str!("graph_state_model.rs");
        let start = SOURCE
            .find("pub struct GraphState {")
            .expect("GraphState declaration exists");
        let body = &SOURCE[start..];
        let end = body.find("\n}").expect("GraphState body terminates");
        &body[..end]
    }

    fn declared_field_line(field: &str) -> &'static str {
        let needles = [format!("{field}:"), format!("pub {field}:")];
        let matches: Vec<&str> = graph_struct_body()
            .lines()
            .map(str::trim)
            .filter(|line| {
                needles
                    .iter()
                    .any(|needle| line.starts_with(needle.as_str()))
            })
            .collect();
        assert_eq!(
            matches.len(),
            1,
            "expected exactly one declaration of {field}"
        );
        matches[0]
    }

    #[test]
    fn test_graph_state_declares_only_its_own_field_private() {
        for field in GRAPH_FIELDS {
            let declaration = declared_field_line(field);
            assert!(
                !declaration.starts_with("pub"),
                "{field} must stay private so no sibling handle can read it"
            );
        }
        assert_eq!(
            graph_struct_body()
                .lines()
                .map(str::trim)
                .filter(|line| line.ends_with(','))
                .count(),
            GRAPH_FIELDS.len(),
            "GraphState may not carry a sibling handle field"
        );
    }

    #[test]
    fn test_read_path_keeps_arcswap_and_adds_no_lock() {
        let body = graph_struct_body();

        assert!(
            declared_field_line("graph").starts_with("graph: ArcSwap<"),
            "graph must stay an ArcSwap snapshot"
        );
        assert!(!body.contains("Mutex"), "no Mutex may wrap the graph state");
        assert!(
            !body.contains("RwLock"),
            "no RwLock may wrap the graph state"
        );
    }

    #[test]
    fn test_reads_return_arcswap_guards() {
        let state = GraphState::new();

        let snapshot: Guard<Arc<GraphSnapshot>> = state.snapshot();

        assert_eq!(snapshot.0.node_count(), 0);
        assert!(snapshot.1.is_empty());
    }

    #[test]
    fn test_handle_owns_the_replaced_snapshot() {
        let state = GraphState::new();

        state.replace(Arc::new(sample_snapshot("wiki:concepts:demo")));

        let snapshot = state.snapshot();
        assert_eq!(snapshot.0.node_count(), 1);
        assert!(snapshot.1.contains_key("wiki:concepts:demo"));
    }

    #[test]
    fn test_update_replaces_the_owned_snapshot() {
        let state = GraphState::new();

        state.update(|current| {
            let (mut graph, mut index) = current.clone();
            let node = graph.add_node(sample_meta("wiki:concepts:demo"));
            index.insert("wiki:concepts:demo".to_string(), node);
            (graph, index)
        });

        assert_eq!(state.snapshot().0.node_count(), 1);
    }

    #[test]
    fn test_separate_handles_do_not_share_state() {
        let first = GraphState::new();
        let second = GraphState::new();
        first.replace(Arc::new(sample_snapshot("wiki:concepts:demo")));

        assert_eq!(first.snapshot().0.node_count(), 1);
        assert_eq!(second.snapshot().0.node_count(), 0);
    }

    #[test]
    fn test_rebuild_fills_the_owned_snapshot() {
        let dir = tempfile::tempdir().expect("temp dir");
        let wiki_dir = dir.path().join(".wm").join("wiki");
        let concepts = wiki_dir.join("concepts");
        std::fs::create_dir_all(&concepts).expect("create wiki dir");
        std::fs::write(
            concepts.join("demo.md"),
            "---\ntitle: Demo\ntype: concept\n---\n\n## Body\n\ncontent\n",
        )
        .expect("write page");
        let state = GraphState::new();

        let count = state.rebuild(&wiki_dir, &[]);

        assert_eq!(count, 1);
        assert_eq!(state.snapshot().0.node_count(), 1);
    }
}
