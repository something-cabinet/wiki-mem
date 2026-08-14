//! Spec item 2 acceptance tests (graphify-gap-closure):
//! AC-2.1 — Rust + TS fixtures with a known call chain produce `calls` edges
//!          with correct file:line and provenance.
//! AC-2.3 — Re-extraction after one file edit is incremental (only the edited
//!          file's edges change).

use std::fs;
use std::path::Path;

use wm_code_intel::services::code_index_db::CodeIndexDb;
use wm_code_intel::services::graph_resolver::{resolve_code_edges, CodeIndexSnapshot};
use wm_code_intel::services::ingest_service::rebuild_code_index;
use wm_engine::models::edge_type_model::EdgeProvenance;

fn open_db(root: &Path) -> CodeIndexDb {
    let db_path = root.join(".wm").join("state").join("code.db");
    fs::create_dir_all(db_path.parent().unwrap()).unwrap();
    CodeIndexDb::open(db_path).unwrap()
}

fn create_source(dir: &Path, rel_path: &str, content: &str) {
    let full = dir.join(rel_path);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(&full, content).unwrap();
}

fn resolved_edges(root: &Path) -> Vec<wm_code_intel::services::graph_resolver::ResolvedCodeEdge> {
    let db = open_db(root);
    let snapshot = CodeIndexSnapshot::from_db(&db).expect("load snapshot");
    resolve_code_edges(&snapshot)
}

#[test]
fn ac21_rust_call_chain_edges_with_line_and_provenance() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    create_source(root, "src/lib.rs", "pub fn helper() -> u32 { 42 }\n");
    create_source(
        root,
        "src/main.rs",
        r#"
use crate::lib::helper;

pub fn caller() {
    helper();
}
"#,
    );

    let db = open_db(root);
    rebuild_code_index(&db, root, false).expect("index");

    let resolved = resolved_edges(root);

    // calls: main.rs caller → lib.rs helper at the `helper();` call site.
    let call = resolved
        .iter()
        .find(|e| e.edge_type == "calls" && e.source_symbol.as_deref() == Some("caller"))
        .unwrap_or_else(|| panic!("expected a calls edge from caller, got {:?}", resolved));
    assert_eq!(call.source_file, "src/main.rs");
    assert_eq!(call.target_file, "src/lib.rs");
    assert_eq!(call.target_symbol.as_deref(), Some("helper"));
    assert_eq!(call.line, 5, "helper() call is on line 5 of src/main.rs");
    assert_eq!(
        call.provenance,
        EdgeProvenance::Explicit,
        "single-candidate symbol resolution is explicit"
    );

    // imports: main.rs → lib.rs (crate::lib::helper).
    let imp = resolved
        .iter()
        .find(|e| e.edge_type == "imports")
        .unwrap_or_else(|| panic!("expected an imports edge, got {:?}", resolved));
    assert_eq!(imp.source_file, "src/main.rs");
    assert_eq!(imp.target_file, "src/lib.rs");
    assert_eq!(imp.line, 2, "use statement is on line 2");
    assert_eq!(imp.provenance, EdgeProvenance::Explicit);
}

#[test]
fn ac21_rust_reexport_chase_produces_derived_provenance() {
    // Real-fixture counterpart of graph_resolver's synthetic
    // `import_through_reexport_is_derived`: the re-export edge must be produced
    // by REAL extraction — `extract_edges` has to capture the plain `argument`
    // of `pub use crate::bar::Bar;` (not the `pub`-prefixed whole node) so the
    // resolver's `chase_reexport` can see it and mark the edge `Derived`.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // src/foo.rs re-exports Bar from src/bar.rs; src/main.rs imports via foo.
    create_source(root, "src/bar.rs", "pub struct Bar;\n");
    create_source(root, "src/foo.rs", "pub use crate::bar::Bar;\n");
    create_source(
        root,
        "src/main.rs",
        r#"
use crate::foo::Bar;

pub fn take_bar(_b: Bar) {}
"#,
    );

    let db = open_db(root);
    rebuild_code_index(&db, root, false).expect("index");

    let resolved = resolved_edges(root);

    let imp = resolved
        .iter()
        .find(|e| e.edge_type == "imports" && e.source_file == "src/main.rs")
        .unwrap_or_else(|| {
            panic!(
                "expected an imports edge from src/main.rs, got {:?}",
                resolved
            )
        });
    assert_eq!(
        imp.target_file, "src/bar.rs",
        "resolution chased through the re-export to the defining file"
    );
    assert_eq!(
        imp.provenance,
        EdgeProvenance::Derived,
        "re-export indirection promotes provenance to Derived"
    );
    assert_eq!(
        imp.via,
        vec!["src/foo.rs".to_string()],
        "via records the re-exporting file"
    );
}

#[test]
fn ac21_ts_call_chain_edges_with_line_and_provenance() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    create_source(
        root,
        "src/utils.ts",
        "export function helper(): number { return 42; }\n",
    );
    create_source(
        root,
        "src/main.ts",
        r#"
import { helper } from './utils';

export function caller(): number {
    return helper();
}
"#,
    );

    let db = open_db(root);
    rebuild_code_index(&db, root, false).expect("index");

    let resolved = resolved_edges(root);

    let call = resolved
        .iter()
        .find(|e| e.edge_type == "calls" && e.source_symbol.as_deref() == Some("caller"))
        .unwrap_or_else(|| panic!("expected a calls edge from caller, got {:?}", resolved));
    assert_eq!(call.source_file, "src/main.ts");
    assert_eq!(call.target_file, "src/utils.ts");
    assert_eq!(call.target_symbol.as_deref(), Some("helper"));
    assert_eq!(call.line, 5, "helper() call is on line 5 of src/main.ts");
    assert_eq!(call.provenance, EdgeProvenance::Explicit);

    let imp = resolved
        .iter()
        .find(|e| e.edge_type == "imports")
        .unwrap_or_else(|| panic!("expected an imports edge, got {:?}", resolved));
    assert_eq!(imp.source_file, "src/main.ts");
    assert_eq!(imp.target_file, "src/utils.ts");
    assert_eq!(imp.provenance, EdgeProvenance::Explicit);
}

#[test]
fn ac23_reextraction_is_incremental_per_file() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    create_source(root, "src/a.rs", "pub fn a_fn() -> u32 { 1 }\n");
    create_source(
        root,
        "src/b.rs",
        r#"
use crate::a::a_fn;
pub fn b_fn() -> u32 { a_fn() }
"#,
    );

    let db = open_db(root);
    let first = rebuild_code_index(&db, root, false).expect("first index");

    let edges_a_before = db
        .query_edges(&wm_code_intel::services::code_index_db::EdgeQuery::forward(Some("src/a.rs")))
        .unwrap();
    let edges_b_before = db
        .query_edges(&wm_code_intel::services::code_index_db::EdgeQuery::forward(Some("src/b.rs")))
        .unwrap();
    assert!(
        first.edges_indexed >= 2,
        "first index stores calls + imports"
    );

    // Edit ONLY src/b.rs: add a second call site.
    create_source(
        root,
        "src/b.rs",
        r#"
use crate::a::a_fn;
pub fn b_fn() -> u32 { a_fn() + a_fn() }
"#,
    );

    let second = rebuild_code_index(&db, root, false).expect("second index");
    assert_eq!(
        second.files_changed, 1,
        "only src/b.rs changed — a.rs hash-skips"
    );
    assert_eq!(
        second.symbols_indexed, 1,
        "only src/b.rs symbols re-extracted"
    );

    let edges_a_after = db
        .query_edges(&wm_code_intel::services::code_index_db::EdgeQuery::forward(Some("src/a.rs")))
        .unwrap();
    let edges_b_after = db
        .query_edges(&wm_code_intel::services::code_index_db::EdgeQuery::forward(Some("src/b.rs")))
        .unwrap();

    assert_eq!(
        edges_a_before, edges_a_after,
        "unmodified file's edges must be untouched (AC-2.3)"
    );
    let call_edges: Vec<_> = edges_b_after
        .iter()
        .filter(|e| e.edge_type == "calls")
        .collect();
    assert_eq!(
        call_edges.len(),
        2,
        "src/b.rs now has 2 call edges (one per a_fn() call site)"
    );
    assert!(
        call_edges.iter().all(|e| e.line == 3),
        "both a_fn() calls sit on line 3 of src/b.rs"
    );
    assert_eq!(
        edges_b_after.len(),
        3,
        "2 calls + 1 import edge in the edited file"
    );
    assert_ne!(edges_b_before, edges_b_after, "edited file's edges change");
}
