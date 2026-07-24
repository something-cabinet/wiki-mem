
use std::path::Path;
use std::sync::Arc;

use arc_swap::ArcSwap;
use petgraph::stable_graph::StableGraph;
use std::collections::HashMap;

use wm_core::engine::{EdgeType, EngineState, GraphSnapshot, WikiPageMeta};
use wm_core::graph;


fn setup_wiki_project() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().to_path_buf();

    let wm_dir = root.join(".wm");
    let wiki_dir = wm_dir.join("wiki");

    for sub in &[
        "tasks", "specs", "concepts", "patterns", "decisions", "howto", "reference", "core",
    ] {
        std::fs::create_dir_all(wiki_dir.join(sub)).expect("create wiki subdir");
    }

    let config = serde_json::json!({
        "schema_version": 1,
        "embedding": { "model_name": "none", "dimensions": 384, "batch_size": 32 },
        "permissions": { "preset": "read-write" },
        "custom_edge_types": [],
        "source_dirs": ["docs/", "specs/"],
        "source_extensions": ["md", "yaml", "txt"],
        "search": {
            "default_mode": "keyword",
            "default_limit": 20,
            "rrf_k": 60,
            "scoring": { "debounce_ms": 500 }
        }
    });
    std::fs::write(
        wm_dir.join("config.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .expect("write config.json");

    (dir, root)
}

fn create_engine(root: &Path) -> (Arc<EngineState>, std::path::PathBuf) {
    use wm_core::config::ProjectConfig;

    let project_root = root.to_path_buf();
    let wiki_dir = root.join(".wm").join("wiki");

    let config = std::fs::read_to_string(root.join(".wm").join("config.json"))
        .ok()
        .and_then(|s| serde_json::from_str::<ProjectConfig>(&s).ok())
        .unwrap_or_default();

    let (engine_state, _audit_rx) = EngineState::new(config, project_root);
    let engine = Arc::new(engine_state);

    engine.rebuild_graph(&wiki_dir);

    (engine, wiki_dir)
}


#[tokio::test(flavor = "multi_thread")]
async fn test_handle_file_change_new_file() {
    let (_dir, root) = setup_wiki_project();
    let (engine, wiki_dir) = create_engine(&root);

    let graph_before = engine.graph.load();
    let nodes_before = graph_before.0.node_count();
    let edges_before = graph_before.0.edge_count();
    drop(graph_before);
    assert_eq!(nodes_before, 0, "expected 0 nodes initially");

    let page_path = wiki_dir.join("concepts").join("watcher-test.md");
    std::fs::write(
        &page_path,
        r#"---
title: Watcher Test
tags: [test, watcher]
status: draft
---

# Watcher Test

Created by direct filesystem write.
"#,
    )
    .expect("write test page");

    graph::handle_file_change(&wiki_dir, &page_path, &engine);

    let graph_after = engine.graph.load();
    let nodes_after = graph_after.0.node_count();
    let edges_after = graph_after.0.edge_count();
    drop(graph_after);

    assert!(
        nodes_after > nodes_before,
        "expected node count to increase after handle_file_change \
         (was {}, now {})",
        nodes_before,
        nodes_after,
    );
    eprintln!(
        "handle_file_change: nodes {} -> {}, edges {} -> {}",
        nodes_before, nodes_after, edges_before, edges_after,
    );

    let snapshot = engine.graph.load();
    let id_index = &snapshot.1;
    let page_id = "wiki:concepts:watcher-test";
    assert!(
        id_index.contains_key(page_id),
        "expected page '{}' to be in graph id_index after handle_file_change",
        page_id,
    );
    drop(snapshot);
}


#[tokio::test(flavor = "multi_thread")]
async fn test_handle_file_change_modification() {
    let (_dir, root) = setup_wiki_project();
    let (engine, wiki_dir) = create_engine(&root);

    let page_path = wiki_dir.join("concepts").join("modify-test.md");
    std::fs::write(
        &page_path,
        r#"---
title: Original Title
tags: [test]
status: draft
---

Original content.
"#,
    )
    .expect("write initial page");

    graph::handle_file_change(&wiki_dir, &page_path, &engine);

    let snapshot_1 = engine.graph.load();
    let nodes_1 = snapshot_1.0.node_count();
    let edges_1 = snapshot_1.0.edge_count();
    drop(snapshot_1);
    assert_eq!(nodes_1, 1, "expected 1 node after creating initial file");

    std::fs::write(
        &page_path,
        r#"---
title: Modified Title
tags: [test, updated]
status: in-progress
---

Modified content with new body tags.
"#,
    )
    .expect("write modified page");

    graph::handle_file_change(&wiki_dir, &page_path, &engine);

    let snapshot_2 = engine.graph.load();
    let nodes_2 = snapshot_2.0.node_count();
    let edges_2 = snapshot_2.0.edge_count();
    drop(snapshot_2);

    assert_eq!(
        nodes_2, nodes_1,
        "node count should remain {} after modifying an existing file",
        nodes_1,
    );

    eprintln!(
        "Modification: nodes {}, edges {} -> {}",
        nodes_2, edges_1, edges_2,
    );

    let custom_types: Vec<String> = Vec::new();
    let rebuilt_count = graph::rebuild_graph_snapshot(&engine.graph, &wiki_dir, &custom_types);
    assert_eq!(
        rebuilt_count, 1,
        "rebuild_graph_snapshot should still report 1 node after modification"
    );
}


#[test]
fn test_build_sections_from_external_file() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path();

    let wiki_dir = root.join(".wm").join("wiki");
    let concepts_dir = wiki_dir.join("concepts");
    std::fs::create_dir_all(&concepts_dir).expect("create concepts dir");

    let page_path = concepts_dir.join("section-test.md");
    std::fs::write(
        &page_path,
        r#"---
title: Section Test
tags: [test]
---

## First Section

This is the first section.

## Second Section

This is the second section with more detail.
"#,
    )
    .expect("write page");

    let sections = graph::build_sections_from_file(&page_path)
        .expect("build_sections_from_file should return Some for valid .md file");

    assert!(
        !sections.is_empty(),
        "expected at least one section from the test file"
    );

    let section_headers: Vec<&str> = sections
        .iter()
        .map(|s| s.header.as_str())
        .collect();
    assert!(
        section_headers.contains(&"First Section"),
        "expected 'First Section' header, got: {:?}",
        section_headers
    );
    assert!(
        section_headers.contains(&"Second Section"),
        "expected 'Second Section' header, got: {:?}",
        section_headers
    );

    let first_section = &sections[0];
    assert!(
        first_section.page_id.contains("concepts:section-test"),
        "page_id should contain 'concepts:section-test', got: {}",
        first_section.page_id
    );

    assert!(
        first_section.tags.contains(&"test".to_string()),
        "tags should include 'test', got: {:?}",
        first_section.tags
    );
}


#[test]
fn test_rebuild_graph_snapshot_direct() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path();

    let wiki_dir = root.join(".wm").join("wiki");
    let concepts_dir = wiki_dir.join("concepts");
    std::fs::create_dir_all(&concepts_dir).expect("create concepts dir");

    std::fs::write(
        concepts_dir.join("graph-test.md"),
        r#"---
title: Graph Test
tags: [test]
---

# Graph Test

A page to test graph rebuild.
"#,
    )
    .expect("write page");

    let graph_swap: ArcSwap<GraphSnapshot> = ArcSwap::new(Arc::new((
        StableGraph::<WikiPageMeta, EdgeType>::new(),
        HashMap::new(),
    )));

    let node_count = graph::rebuild_graph_snapshot(
        &graph_swap,
        &wiki_dir,
        &[],
    );

    assert_eq!(
        node_count, 1,
        "expected 1 node after rebuilding graph from wiki directory with one file"
    );

    let snapshot = graph_swap.load();
    let (graph, id_index) = &**snapshot;
    assert_eq!(graph.node_count(), 1, "graph should have 1 node");
    assert_eq!(id_index.len(), 1, "id_index should have 1 entry");

    let node_id = id_index.keys().next().expect("at least one id");
    assert!(
        node_id.contains("graph-test"),
        "expected node id containing 'graph-test', got: {}",
        node_id
    );

    std::fs::write(
        concepts_dir.join("graph-test-2.md"),
        r#"---
title: Graph Test 2
tags: [test]
---

# Graph Test 2

Second page.
"#,
    )
    .expect("write second page");

    let node_count_2 = graph::rebuild_graph_snapshot(
        &graph_swap,
        &wiki_dir,
        &[],
    );

    assert_eq!(
        node_count_2, 2,
        "expected 2 nodes after adding a second file and rebuilding"
    );

    let snapshot2 = graph_swap.load();
    assert_eq!(snapshot2.0.node_count(), 2);
}


#[tokio::test(flavor = "multi_thread")]
async fn test_handle_file_delete_removes_page() {
    let (_dir, root) = setup_wiki_project();
    let (engine, wiki_dir) = create_engine(&root);

    let page1 = wiki_dir.join("concepts").join("delete-test-1.md");
    let page2 = wiki_dir.join("concepts").join("delete-test-2.md");
    std::fs::write(&page1, "---\ntitle: Delete Test 1\n---\n# Delete Test 1\n").expect("write page1");
    std::fs::write(&page2, "---\ntitle: Delete Test 2\n---\n# Delete Test 2\n").expect("write page2");

    graph::handle_file_change(&wiki_dir, &page1, &engine);
    graph::handle_file_change(&wiki_dir, &page2, &engine);

    let snapshot_before = engine.graph.load();
    let nodes_before = snapshot_before.0.node_count();
    drop(snapshot_before);
    assert_eq!(nodes_before, 2, "expected 2 nodes before delete");

    std::fs::remove_file(&page1).expect("remove page1");
    graph::handle_file_delete(&wiki_dir, &page1, &engine);

    let snapshot_after = engine.graph.load();
    let nodes_after = snapshot_after.0.node_count();
    drop(snapshot_after);

    assert_eq!(
        nodes_after, 1,
        "expected 1 node after deleting one page (was {}, now {})",
        nodes_before, nodes_after,
    );
}
