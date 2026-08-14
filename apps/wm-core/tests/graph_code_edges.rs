//! Spec items 2 + 6 acceptance tests through the real in-process MCP tools:
//! AC-2.2 — `wm_graph.neighbors` on a code symbol returns typed code edges
//!          alongside wiki edges.
//! AC-6.1 — `wm_graph.affected` on a function node lists all transitively
//!          affected symbols with edge paths.
//! AC-6.2 — wiki-page dependencies (`depends_on`, `extends`) are included in
//!          the affected set.

#![cfg(feature = "code-intel")]

use std::fs;
use std::path::Path;

use wm_code_intel::services::code_index_db::CodeIndexDb;
use wm_code_intel::services::ingest_service::rebuild_code_index;

#[path = "helpers/inproc.rs"]
mod inproc;
use inproc::{call_ok, setup_in_process};

fn create_source(dir: &Path, rel_path: &str, content: &str) {
    let full = dir.join(rel_path);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(&full, content).unwrap();
}

fn write_page(dir: &Path, rel_path: &str, content: &str) {
    let full = dir.join(rel_path);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(&full, content).unwrap();
}

fn build_code_index(root: &Path) {
    let db_path = root.join(".wm").join("state").join("code.db");
    fs::create_dir_all(db_path.parent().unwrap()).unwrap();
    let db = CodeIndexDb::open(db_path).unwrap();
    rebuild_code_index(&db, root, false).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn ac22_neighbors_on_code_symbol_returns_typed_edges() {
    let ((_dir, root, _engine, registry), _guard) = setup_in_process().await;

    create_source(&root, "src/lib.rs", "pub fn helper() -> u32 { 42 }\n");
    create_source(
        &root,
        "src/main.rs",
        r#"
use crate::lib::helper;

pub fn caller() -> u32 {
    helper()
}
"#,
    );
    build_code_index(&root);

    let resp = call_ok(
        &registry,
        "wm_graph.neighbors",
        serde_json::json!({ "id": "src/lib.rs#helper" }),
    )
    .await;

    let neighbors = resp["neighbors"].as_array().expect("neighbors array");
    assert!(
        !neighbors.is_empty(),
        "code symbol must return code edges: {:?}",
        resp
    );

    let call = neighbors
        .iter()
        .find(|n| n["edge_type"] == "calls")
        .expect("a calls edge is present");
    assert_eq!(call["id"], "src/main.rs#caller", "caller node id");
    assert_eq!(call["provenance"], "explicit");
    assert_eq!(call["line"].as_u64(), Some(5), "call site is on line 5");
    assert_eq!(call["source_file"], "src/main.rs");
    assert_eq!(call["target_file"], "src/lib.rs");

    let imp = neighbors
        .iter()
        .find(|n| n["edge_type"] == "imports")
        .expect("an imports edge is present");
    assert_eq!(imp["id"], "src/main.rs", "importing file node");
    assert_eq!(imp["provenance"], "explicit");
}

#[tokio::test(flavor = "multi_thread")]
async fn ac61_affected_function_lists_transitive_callers() {
    let ((_dir, root, _engine, registry), _guard) = setup_in_process().await;

    // helper <- run <- main  (transitive call chain)
    create_source(&root, "src/step.rs", "pub fn step() -> u32 { 1 }\n");
    create_source(
        &root,
        "src/engine.rs",
        r#"
use crate::step::step;
pub fn run() -> u32 { step() }
"#,
    );
    create_source(
        &root,
        "src/main.rs",
        r#"
use crate::engine::run;
pub fn main() -> u32 { run() }
"#,
    );
    build_code_index(&root);

    let resp = call_ok(
        &registry,
        "wm_graph.affected",
        serde_json::json!({ "node": "src/step.rs#step" }),
    )
    .await;

    assert_eq!(resp["kind"], "code");
    let affected = resp["affected"].as_array().expect("affected array");
    let ids: Vec<&str> = affected
        .iter()
        .map(|a| a["id"].as_str().unwrap_or(""))
        .collect();
    assert!(
        ids.contains(&"src/engine.rs#run"),
        "run calls step: {:?}",
        ids
    );
    assert!(
        ids.contains(&"src/main.rs#main"),
        "main calls run — transitively affected: {:?}",
        ids
    );

    let main_node = affected
        .iter()
        .find(|a| a["id"] == "src/main.rs#main")
        .expect("main node present");
    let hops = main_node["hops"].as_array().expect("hops array");
    assert_eq!(hops.len(), 2, "main is 2 hops from step");
    assert_eq!(hops[0]["edge_type"], "calls");
    assert_eq!(hops[0]["from"], "src/engine.rs#run");
    assert_eq!(hops[0]["to"], "src/step.rs#step");
    assert_eq!(hops[0]["provenance"], "explicit");
    assert!(hops[0]["line"].is_u64(), "code hops carry file:line");
    assert_eq!(hops[1]["edge_type"], "calls");
    assert_eq!(hops[1]["from"], "src/main.rs#main");
    assert_eq!(hops[1]["to"], "src/engine.rs#run");
}

#[tokio::test(flavor = "multi_thread")]
async fn ac62_affected_includes_wiki_depends_on_extends() {
    let ((_dir, root, _engine, registry), _guard) = setup_in_process().await;

    write_page(
        &root,
        ".wm/wiki/core/db.md",
        "---\ntitle: DB\ntype: core\n---\n\nDB.\n",
    );
    write_page(
        &root,
        ".wm/wiki/core/repo.md",
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
        &root,
        ".wm/wiki/concepts/service.md",
        r#"---
title: Service
type: concept
relates_to:
  - type: extends
    target: wiki:core:repo
---
Service.
"#,
    );

    let _ = call_ok(
        &registry,
        "wm_index_rebuild",
        serde_json::json!({ "skip_embed": true }),
    )
    .await;

    let resp = call_ok(
        &registry,
        "wm_graph.affected",
        serde_json::json!({ "node": "wiki:core:db" }),
    )
    .await;

    assert_eq!(resp["kind"], "page");
    let affected = resp["affected"].as_array().expect("affected array");
    let ids: Vec<&str> = affected
        .iter()
        .map(|a| a["id"].as_str().unwrap_or(""))
        .collect();
    assert!(
        ids.contains(&"wiki:core:repo"),
        "repo depends_on db: {:?}",
        ids
    );
    assert!(
        ids.contains(&"wiki:concepts:service"),
        "service extends repo (→ db) transitively: {:?}",
        ids
    );

    let service = affected
        .iter()
        .find(|a| a["id"] == "wiki:concepts:service")
        .expect("service node present");
    let hops = service["hops"].as_array().expect("hops array");
    assert_eq!(hops.len(), 2);
    assert_eq!(hops[0]["edge_type"], "depends_on");
    assert_eq!(hops[0]["from"], "wiki:core:repo");
    assert_eq!(hops[0]["to"], "wiki:core:db");
    assert_eq!(hops[1]["edge_type"], "extends");
    assert_eq!(hops[1]["from"], "wiki:concepts:service");
    assert_eq!(hops[1]["to"], "wiki:core:repo");
}

// FR-2.3: typed code edges are exposed through wm_code.deps / wm_code.search
// with provenance and file:line.

#[tokio::test(flavor = "multi_thread")]
async fn fr23_code_deps_returns_typed_edges_with_provenance() {
    let ((_dir, root, _engine, registry), _guard) = setup_in_process().await;

    create_source(&root, "src/lib.rs", "pub fn helper() -> u32 { 42 }\n");
    create_source(
        &root,
        "src/main.rs",
        r#"
use crate::lib::helper;

pub fn caller() -> u32 {
    helper()
}
"#,
    );

    let resp = call_ok(
        &registry,
        "wm_code.deps",
        serde_json::json!({ "file": "src/main.rs" }),
    )
    .await;

    let deps = resp["dependencies"].as_array().expect("dependencies array");
    let main_entry = deps
        .iter()
        .find(|d| d["file"].as_str().unwrap_or("").contains("src/main.rs"))
        .expect("main.rs entry present");
    let edges = main_entry["edges"]
        .as_array()
        .expect("edges array on entry");
    let calls = edges
        .iter()
        .find(|e| e["edge_type"] == "calls")
        .expect("calls edge present");
    assert_eq!(calls["target_file"], "src/lib.rs");
    assert_eq!(calls["target_symbol"], "helper");
    assert_eq!(calls["provenance"], "explicit");
    assert_eq!(calls["line"].as_u64(), Some(5));
}

#[tokio::test(flavor = "multi_thread")]
async fn fr23_code_search_edge_type_returns_edges() {
    let ((_dir, root, _engine, registry), _guard) = setup_in_process().await;

    create_source(&root, "src/lib.rs", "pub fn helper() -> u32 { 42 }\n");
    create_source(
        &root,
        "src/main.rs",
        r#"
use crate::lib::helper;

pub fn caller() -> u32 {
    helper()
}
"#,
    );

    let resp = call_ok(
        &registry,
        "wm_code.search",
        serde_json::json!({ "pattern": "helper", "edge_type": "calls" }),
    )
    .await;

    let edges = resp["edges"].as_array().expect("edges array");
    assert!(
        !edges.is_empty(),
        "edge search returns matching calls edges"
    );
    let e = &edges[0];
    assert_eq!(e["edge_type"], "calls");
    assert_eq!(e["target_symbol"], "helper");
    assert_eq!(e["source_file"], "src/main.rs");
    assert_eq!(e["line"].as_u64(), Some(5));
    assert_eq!(e["provenance"], "explicit");
}
