// ─── P5a File Watcher Integration Tests ───────────────────────
//
// Tests that writing an .md file directly to .wm/wiki/ via the
// filesystem (not via MCP tools) causes the graph to update via
// the `handle_file_change` / `rebuild_graph_snapshot` pipeline.
//
// Strategy (the file watcher runs in a background tokio thread tied
// to the engine lifecycle, so the most practical approach is):
//   1. Create a temp wiki directory with an EngineState.
//   2. Call rebuild_graph_snapshot to initialize the graph.
//   3. Write a new .md file via std::fs::write.
//   4. Call handle_file_change directly (simulating what the
//      notify debouncer would do).
//   5. Call rebuild_graph_snapshot again and verify the graph
//      now contains the new page.
//
// We also include direct unit checks for build_sections_from_file
// and rebuild_graph_snapshot in isolation.

use std::path::Path;
use std::sync::Arc;

use arc_swap::ArcSwap;
use petgraph::stable_graph::StableGraph;
use std::collections::HashMap;

use wm_core::engine::{EdgeType, EngineState, GraphSnapshot, WikiPageMeta};
use wm_core::graph;

// ─── Helpers ─────────────────────────────────────────────────

/// Create a temporary wiki project directory with the minimal
/// structure (.wm/wiki/ + subdirs, .wm/config.json).
fn setup_wiki_project() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().to_path_buf();

    let wm_dir = root.join(".wm");
    let wiki_dir = wm_dir.join("wiki");

    // Create wiki subdirectory structure (must match setup_test_project)
    for sub in &[
        "tasks", "specs", "concepts", "patterns", "decisions", "howto", "reference",
    ] {
        std::fs::create_dir_all(wiki_dir.join(sub)).expect("create wiki subdir");
    }

    // Write a minimal config.json with default settings
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

/// Create an EngineState in a tokio context.
/// Returns the engine + wiki_dir for the test.
fn create_engine(root: &Path) -> (Arc<EngineState>, std::path::PathBuf) {
    use wm_core::config::ProjectConfig;

    let project_root = root.to_path_buf();
    let wiki_dir = root.join(".wm").join("wiki");

    // Load config from disk or use defaults
    let config = std::fs::read_to_string(root.join(".wm").join("config.json"))
        .ok()
        .and_then(|s| serde_json::from_str::<ProjectConfig>(&s).ok())
        .unwrap_or_default();

    let (engine_state, _audit_rx) = EngineState::new(config, project_root);
    let engine = Arc::new(engine_state);

    // Initial rebuild to populate the graph
    engine.rebuild_graph(&wiki_dir);

    (engine, wiki_dir)
}

// ═══════════════════════════════════════════════════════════════
// P5a-1: handle_file_change picks up a newly created .md file
// ═══════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn test_handle_file_change_new_file() {
    let (_dir, root) = setup_wiki_project();
    let (engine, wiki_dir) = create_engine(&root);

    // ── Snapshot initial graph state (should be 0 nodes) ──
    let graph_before = engine.graph.load();
    let nodes_before = graph_before.0.node_count();
    let edges_before = graph_before.0.edge_count();
    drop(graph_before);
    assert_eq!(nodes_before, 0, "expected 0 nodes initially");

    // ── Write a new .md file to the wiki via std::fs ──────
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

    // ── Simulate file watcher: call handle_file_change ─────
    graph::handle_file_change(&wiki_dir, &page_path, &engine);

    // ── Verify the graph now contains the new page ─────────
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

    // ── Verify the page is in the id_index ─────────────────
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

// ═══════════════════════════════════════════════════════════════
// P5a-2: handle_file_change detects modifications to .md files
// ═══════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn test_handle_file_change_modification() {
    let (_dir, root) = setup_wiki_project();
    let (engine, wiki_dir) = create_engine(&root);

    // ── Write an initial .md file ─────────────────────────
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

    // Process it
    graph::handle_file_change(&wiki_dir, &page_path, &engine);

    // Verify it was picked up
    let snapshot_1 = engine.graph.load();
    let nodes_1 = snapshot_1.0.node_count();
    let edges_1 = snapshot_1.0.edge_count();
    drop(snapshot_1);
    assert_eq!(nodes_1, 1, "expected 1 node after creating initial file");

    // ── Modify the file via std::fs ───────────────────────
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

    // Process the modification
    graph::handle_file_change(&wiki_dir, &page_path, &engine);

    // ── Verify graph state after modification ─────────────
    let snapshot_2 = engine.graph.load();
    let nodes_2 = snapshot_2.0.node_count();
    let edges_2 = snapshot_2.0.edge_count();
    drop(snapshot_2);

    // Node count should stay the same (we modified, not added)
    assert_eq!(
        nodes_2, nodes_1,
        "node count should remain {} after modifying an existing file",
        nodes_1,
    );

    // Edges may change (new tags could create new relationships)
    eprintln!(
        "Modification: nodes {}, edges {} -> {}",
        nodes_2, edges_1, edges_2,
    );

    // ── Also verify via rebuild_graph_snapshot ────────────
    let custom_types: Vec<String> = Vec::new();
    let rebuilt_count = graph::rebuild_graph_snapshot(&engine.graph, &wiki_dir, &custom_types);
    assert_eq!(
        rebuilt_count, 1,
        "rebuild_graph_snapshot should still report 1 node after modification"
    );
}

// ═══════════════════════════════════════════════════════════════
// P5a-3: build_sections_from_file works with externally-created file
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_build_sections_from_external_file() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path();

    // Set up minimal wiki directory structure
    let wiki_dir = root.join(".wm").join("wiki");
    let concepts_dir = wiki_dir.join("concepts");
    std::fs::create_dir_all(&concepts_dir).expect("create concepts dir");

    // Write a page via std::fs (simulating external editor)
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

    // Call build_sections_from_file directly
    let sections = graph::build_sections_from_file(&page_path)
        .expect("build_sections_from_file should return Some for valid .md file");

    assert!(
        !sections.is_empty(),
        "expected at least one section from the test file"
    );

    // Verify section headers
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

    // Verify page_id derivation
    let first_section = &sections[0];
    assert!(
        first_section.page_id.contains("concepts:section-test"),
        "page_id should contain 'concepts:section-test', got: {}",
        first_section.page_id
    );

    // Verify tags flow through
    assert!(
        first_section.tags.contains(&"test".to_string()),
        "tags should include 'test', got: {:?}",
        first_section.tags
    );
}

// ═══════════════════════════════════════════════════════════════
// P5a-4: rebuild_graph_snapshot picks up files written by std::fs
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_rebuild_graph_snapshot_direct() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path();

    // Set up wiki directory and write a page
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

    // Create a fresh graph swap
    let graph_swap: ArcSwap<GraphSnapshot> = ArcSwap::new(Arc::new((
        StableGraph::<WikiPageMeta, EdgeType>::new(),
        HashMap::new(),
    )));

    // Rebuild from the wiki directory
    let node_count = graph::rebuild_graph_snapshot(
        &graph_swap,
        &wiki_dir,
        &[], // no custom edge types
    );

    assert_eq!(
        node_count, 1,
        "expected 1 node after rebuilding graph from wiki directory with one file"
    );

    // Verify the snapshot contains the page
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

    // ── Write a second page and rebuild again ──
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

// ═══════════════════════════════════════════════════════════════
// P5a-5: handle_file_delete removes a page from the graph
// ═══════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn test_handle_file_delete_removes_page() {
    let (_dir, root) = setup_wiki_project();
    let (engine, wiki_dir) = create_engine(&root);

    // Write two pages
    let page1 = wiki_dir.join("concepts").join("delete-test-1.md");
    let page2 = wiki_dir.join("concepts").join("delete-test-2.md");
    std::fs::write(&page1, "---\ntitle: Delete Test 1\n---\n# Delete Test 1\n").expect("write page1");
    std::fs::write(&page2, "---\ntitle: Delete Test 2\n---\n# Delete Test 2\n").expect("write page2");

    // Process both
    graph::handle_file_change(&wiki_dir, &page1, &engine);
    graph::handle_file_change(&wiki_dir, &page2, &engine);

    let snapshot_before = engine.graph.load();
    let nodes_before = snapshot_before.0.node_count();
    drop(snapshot_before);
    assert_eq!(nodes_before, 2, "expected 2 nodes before delete");

    // Delete one page
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
