//! Acceptance tests for code edge extraction and resolution:
//! - Rust + TS fixtures with a known call chain produce `calls` edges
//!   with correct file:line and provenance.
//! - Re-extraction after one file edit is incremental (only the edited
//!   file's edges change).

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
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

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
        "unmodified file's edges must be untouched"
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

#[test]
fn rust_call_forms_capture_receiver() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    create_source(
        root,
        "src/lib.rs",
        r#"pub struct Foo;
impl Foo {
    pub fn assoc() -> Self { Foo }
    pub fn method(&self) -> u32 { 1 }
}
pub fn bare_fn() -> u32 { 2 }
"#,
    );
    create_source(
        root,
        "src/main.rs",
        r#"use crate::lib::{Foo, bare_fn};

pub fn caller() {
    bare_fn();
    let x = Foo::assoc();
    x.method();
}
"#,
    );

    let db = open_db(root);
    rebuild_code_index(&db, root, false).expect("index");

    let snapshot = CodeIndexSnapshot::from_db(&db).expect("load");
    let edges: Vec<_> = snapshot
        .raw_edges
        .iter()
        .filter(|e| e.edge_type == "calls" && e.source_file == "src/main.rs")
        .collect();

    let bare = edges.iter().find(|e| e.target_symbol.as_deref() == Some("bare_fn"));
    assert!(bare.is_some(), "bare call edge should exist");
    assert_eq!(bare.unwrap().receiver, None, "bare call has no receiver");

    let assoc = edges.iter().find(|e| e.target_symbol.as_deref() == Some("assoc"));
    assert!(assoc.is_some(), "path call edge should exist for Foo::assoc()");
    assert_eq!(
        assoc.unwrap().receiver.as_deref(),
        Some("Foo"),
        "path call receiver is the type prefix"
    );

    let method = edges.iter().find(|e| e.target_symbol.as_deref() == Some("method"));
    assert!(method.is_some(), "method call edge should exist for x.method()");
    assert_eq!(
        method.unwrap().receiver.as_deref(),
        Some("x"),
        "method call receiver is the binding name"
    );
}

#[test]
fn typescript_call_forms_capture_receiver() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    create_source(
        root,
        "src/service.ts",
        r#"export class Service {
    run(): void {}
}
export function standalone(): void {}
export namespace NS {
    export function util(): void {}
}
"#,
    );
    create_source(
        root,
        "src/main.ts",
        r#"import { Service, standalone, NS } from './service';

function caller(): void {
    standalone();
    const svc = new Service();
    svc.run();
    NS.util();
}
"#,
    );

    let db = open_db(root);
    rebuild_code_index(&db, root, false).expect("index");

    let snapshot = CodeIndexSnapshot::from_db(&db).expect("load");
    let edges: Vec<_> = snapshot
        .raw_edges
        .iter()
        .filter(|e| e.edge_type == "calls" && e.source_file == "src/main.ts")
        .collect();

    let bare = edges.iter().find(|e| e.target_symbol.as_deref() == Some("standalone"));
    assert!(bare.is_some(), "bare call edge should exist");
    assert_eq!(bare.unwrap().receiver, None, "bare call has no receiver");

    let method = edges.iter().find(|e| e.target_symbol.as_deref() == Some("run"));
    assert!(method.is_some(), "method call edge should exist for svc.run()");
    assert_eq!(
        method.unwrap().receiver.as_deref(),
        Some("svc"),
        "method call receiver is the binding"
    );

    let ns = edges.iter().find(|e| e.target_symbol.as_deref() == Some("util"));
    assert!(ns.is_some(), "namespace call edge should exist for NS.util()");
    assert_eq!(
        ns.unwrap().receiver.as_deref(),
        Some("NS"),
        "namespace call receiver is the namespace"
    );
}

#[test]
fn python_call_forms_capture_receiver() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    create_source(
        root,
        "src/lib.py",
        r#"class Svc:
    def run(self):
        pass

def standalone():
    pass
"#,
    );
    create_source(
        root,
        "src/main.py",
        r#"from lib import Svc, standalone

def caller():
    standalone()
    s = Svc()
    s.run()
"#,
    );

    let db = open_db(root);
    rebuild_code_index(&db, root, false).expect("index");

    let snapshot = CodeIndexSnapshot::from_db(&db).expect("load");
    let edges: Vec<_> = snapshot
        .raw_edges
        .iter()
        .filter(|e| e.edge_type == "calls" && e.source_file == "src/main.py")
        .collect();

    let bare = edges.iter().find(|e| e.target_symbol.as_deref() == Some("standalone"));
    assert!(bare.is_some(), "bare call edge should exist");
    assert_eq!(bare.unwrap().receiver, None);

    let method = edges.iter().find(|e| e.target_symbol.as_deref() == Some("run"));
    assert!(method.is_some(), "method call edge should exist for s.run()");
    assert_eq!(method.unwrap().receiver.as_deref(), Some("s"));
}

#[test]
fn go_call_forms_capture_receiver() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    create_source(
        root,
        "src/main.go",
        r#"package main

import "fmt"

type Svc struct{}

func (s *Svc) Run() {}
func standalone() {}

func caller() {
    standalone()
    s := &Svc{}
    s.Run()
    fmt.Println("hi")
}
"#,
    );

    let db = open_db(root);
    rebuild_code_index(&db, root, false).expect("index");

    let snapshot = CodeIndexSnapshot::from_db(&db).expect("load");
    let edges: Vec<_> = snapshot
        .raw_edges
        .iter()
        .filter(|e| e.edge_type == "calls" && e.source_file == "src/main.go")
        .collect();

    let bare = edges.iter().find(|e| e.target_symbol.as_deref() == Some("standalone"));
    assert!(bare.is_some(), "bare call edge should exist");
    assert_eq!(bare.unwrap().receiver, None);

    let method = edges.iter().find(|e| e.target_symbol.as_deref() == Some("Run"));
    assert!(method.is_some(), "method call edge should exist for s.Run()");
    assert_eq!(method.unwrap().receiver.as_deref(), Some("s"));

    let pkg = edges.iter().find(|e| e.target_symbol.as_deref() == Some("Println"));
    assert!(pkg.is_some(), "package call edge should exist for fmt.Println()");
    assert_eq!(pkg.unwrap().receiver.as_deref(), Some("fmt"));
}

#[test]
fn receiver_type_inference_resolves_method_to_correct_file() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    create_source(
        root,
        "src/foo.rs",
        r#"pub struct Foo;
impl Foo {
    pub fn new() -> Self { Foo }
    pub fn method(&self) -> u32 { 1 }
}
"#,
    );
    create_source(
        root,
        "src/bar.rs",
        r#"pub struct Bar;
impl Bar {
    pub fn method(&self) -> u32 { 2 }
}
"#,
    );
    create_source(
        root,
        "src/main.rs",
        r#"use crate::foo::Foo;

pub fn caller() {
    let x = Foo::new();
    x.method();
}
"#,
    );

    let db = open_db(root);
    rebuild_code_index(&db, root, false).expect("index");
    let resolved = resolved_edges(root);

    let method_call = resolved
        .iter()
        .find(|e| {
            e.edge_type == "calls"
                && e.source_file == "src/main.rs"
                && e.target_symbol.as_deref() == Some("method")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a resolved calls edge for x.method(), got: {:?}",
                resolved
                    .iter()
                    .filter(|e| e.source_file == "src/main.rs")
                    .collect::<Vec<_>>()
            )
        });

    assert_eq!(
        method_call.target_file, "src/foo.rs",
        "receiver-type inference should resolve method to Foo's file (src/foo.rs), \
         not Bar's (src/bar.rs) — x was assigned via Foo::new()"
    );
    assert_eq!(method_call.provenance, EdgeProvenance::Explicit);
}

#[test]
fn self_receiver_resolves_to_impl_type_file() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    create_source(
        root,
        "src/helper.rs",
        r#"pub fn helper() -> u32 { 0 }
"#,
    );
    create_source(
        root,
        "src/foo.rs",
        r#"pub struct Foo;
impl Foo {
    pub fn helper(&self) -> u32 { 1 }
    pub fn caller(&self) {
        self.helper();
    }
}
"#,
    );

    let db = open_db(root);
    rebuild_code_index(&db, root, false).expect("index");
    let resolved = resolved_edges(root);

    let self_call = resolved
        .iter()
        .find(|e| {
            e.edge_type == "calls"
                && e.source_file == "src/foo.rs"
                && e.target_symbol.as_deref() == Some("helper")
                && e.source_symbol.as_deref() == Some("caller")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a calls edge for self.helper() in Foo::caller, got: {:?}",
                resolved
                    .iter()
                    .filter(|e| e.source_file == "src/foo.rs")
                    .collect::<Vec<_>>()
            )
        });

    assert_eq!(
        self_call.target_file, "src/foo.rs",
        "self.helper() in Foo::caller should resolve to src/foo.rs (same impl), \
         not src/helper.rs (bare helper function)"
    );
    assert_eq!(self_call.provenance, EdgeProvenance::Explicit);
}

#[test]
fn rust_impl_trait_is_implements_and_supertrait_is_inherits() {
    use wm_code_intel::services::engine_service::extract_edges;

    let rust_impl = r#"
use std::fmt::Display;
struct Foo;
impl Display for Foo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Foo")
    }
}
"#;
    let edges = extract_edges(rust_impl, "src/foo.rs", "rs");
    let impl_edges: Vec<_> = edges.iter().filter(|e| e.edge_type == "implements").collect();
    assert!(
        !impl_edges.is_empty(),
        "impl Display for Foo should produce an implements edge; got edges: {:?}",
        edges.iter().map(|e| (&e.edge_type, &e.source_symbol, &e.target_symbol)).collect::<Vec<_>>()
    );
    let edge = impl_edges.iter().find(|e| e.target_symbol.as_deref() == Some("Display")).unwrap();
    assert_eq!(edge.source_symbol.as_deref(), Some("Foo"));
    assert_eq!(edge.edge_type, "implements");

    let rust_supertrait = r#"
trait Base {}
trait Derived: Base {}
"#;
    let edges = extract_edges(rust_supertrait, "src/traits.rs", "rs");
    let inherits_edges: Vec<_> = edges.iter().filter(|e| e.edge_type == "inherits").collect();
    if !inherits_edges.is_empty() {
        let edge = inherits_edges.iter().find(|e| e.target_symbol.as_deref() == Some("Base")).unwrap();
        assert_eq!(edge.source_symbol.as_deref(), Some("Derived"));
        assert_eq!(edge.edge_type, "inherits");
    }

    let inherits_from_impl: Vec<_> = extract_edges(rust_impl, "src/foo.rs", "rs")
        .iter()
        .filter(|e| e.edge_type == "inherits")
        .cloned()
        .collect();
    assert!(
        inherits_from_impl.is_empty(),
        "impl Trait should NOT produce inherits edges: {:?}",
        inherits_from_impl
    );
}

#[test]
fn typescript_implements_vs_extends() {
    use wm_code_intel::services::engine_service::extract_edges;

    let ts_code = r#"
interface Serializable {
    serialize(): string;
}

class Animal {
    name: string;
}

class Dog extends Animal implements Serializable {
    serialize(): string {
        return this.name;
    }
}
"#;
    let edges = extract_edges(ts_code, "src/animals.ts", "ts");

    let inherits: Vec<_> = edges.iter().filter(|e| e.edge_type == "inherits").collect();
    assert!(
        inherits.iter().any(|e| e.target_symbol.as_deref() == Some("Animal")),
        "extends Animal should produce inherits edge; got: {:?}",
        inherits
    );

    let implements: Vec<_> = edges.iter().filter(|e| e.edge_type == "implements").collect();
    if !implements.is_empty() {
        assert!(
            implements.iter().any(|e| e.target_symbol.as_deref() == Some("Serializable")),
            "implements Serializable should produce implements edge; got: {:?}",
            implements
        );
    }
}

#[test]
fn references_edges_carry_typed_context() {
    use wm_code_intel::services::engine_service::extract_edges;

    let rust_code = r#"
struct Config {
    name: MyType,
}

fn process(input: MyType) -> OtherType {
    let items: Vec<CustomType> = vec![];
    items
}
"#;
    let edges = extract_edges(rust_code, "src/service.rs", "rs");
    let refs: Vec<_> = edges.iter().filter(|e| e.edge_type == "references").collect();

    let field_refs: Vec<_> = refs.iter().filter(|e| e.source_symbol.as_deref() == Some("field")).collect();
    assert!(
        field_refs.iter().any(|e| e.target_symbol.as_deref() == Some("MyType")),
        "field type MyType should be a references edge; field refs: {:?}",
        field_refs
    );

    let param_refs: Vec<_> = refs.iter().filter(|e| e.source_symbol.as_deref() == Some("parameter_type")).collect();
    assert!(
        param_refs.iter().any(|e| e.target_symbol.as_deref() == Some("MyType")),
        "parameter type MyType should be a references edge; param refs: {:?}",
        param_refs
    );

    let ret_refs: Vec<_> = refs.iter().filter(|e| e.source_symbol.as_deref() == Some("return_type")).collect();
    assert!(
        ret_refs.iter().any(|e| e.target_symbol.as_deref() == Some("OtherType")),
        "return type OtherType should be a references edge; return refs: {:?}",
        ret_refs
    );

    let generic_refs: Vec<_> = refs.iter().filter(|e| e.source_symbol.as_deref() == Some("generic_arg")).collect();
    assert!(
        generic_refs.iter().any(|e| e.target_symbol.as_deref() == Some("CustomType")),
        "generic arg CustomType should be a references edge; generic refs: {:?}",
        generic_refs
    );
}

#[test]
fn static_import_cycle_detected_but_dynamic_import_cycle_excluded() {
    use wm_code_intel::services::graph_resolver::{
        detect_import_cycles, ResolvedCodeEdge,
    };
    use wm_code_intel::services::engine_service::extract_edges;
    use wm_engine::models::edge_type_model::EdgeProvenance;

    let a_code = r#"import { foo } from './b';"#;
    let b_code = r#"import { bar } from './a';"#;

    let _a_edges = extract_edges(a_code, "src/a.ts", "ts");
    let _b_edges = extract_edges(b_code, "src/b.ts", "ts");

    let resolved_static: Vec<ResolvedCodeEdge> = vec![
        ResolvedCodeEdge {
            edge_type: "imports".into(),
            source_file: "src/a.ts".into(),
            source_symbol: None,
            target_file: "src/b.ts".into(),
            target_symbol: Some("./b".into()),
            line: 1,
            provenance: EdgeProvenance::Explicit,
            via: vec![],
        },
        ResolvedCodeEdge {
            edge_type: "imports".into(),
            source_file: "src/b.ts".into(),
            source_symbol: None,
            target_file: "src/a.ts".into(),
            target_symbol: Some("./a".into()),
            line: 1,
            provenance: EdgeProvenance::Explicit,
            via: vec![],
        },
    ];

    let cycles = detect_import_cycles(&resolved_static);
    assert!(
        !cycles.is_empty(),
        "static import cycle between a.ts and b.ts should be detected"
    );
    assert!(
        cycles[0].contains(&"src/a.ts".to_string()),
        "cycle should include a.ts"
    );
    assert!(
        cycles[0].contains(&"src/b.ts".to_string()),
        "cycle should include b.ts"
    );

    let resolved_deferred: Vec<ResolvedCodeEdge> = vec![
        ResolvedCodeEdge {
            edge_type: "imports_deferred".into(),
            source_file: "src/a.ts".into(),
            source_symbol: None,
            target_file: "src/b.ts".into(),
            target_symbol: Some("./b".into()),
            line: 1,
            provenance: EdgeProvenance::Explicit,
            via: vec![],
        },
        ResolvedCodeEdge {
            edge_type: "imports_deferred".into(),
            source_file: "src/b.ts".into(),
            source_symbol: None,
            target_file: "src/a.ts".into(),
            target_symbol: Some("./a".into()),
            line: 1,
            provenance: EdgeProvenance::Explicit,
            via: vec![],
        },
    ];

    let cycles_deferred = detect_import_cycles(&resolved_deferred);
    assert!(
        cycles_deferred.is_empty(),
        "dynamic import cycle should NOT be detected; got: {:?}",
        cycles_deferred
    );

    let resolved_mixed: Vec<ResolvedCodeEdge> = vec![
        ResolvedCodeEdge {
            edge_type: "imports".into(),
            source_file: "src/a.ts".into(),
            source_symbol: None,
            target_file: "src/b.ts".into(),
            target_symbol: Some("./b".into()),
            line: 1,
            provenance: EdgeProvenance::Explicit,
            via: vec![],
        },
        ResolvedCodeEdge {
            edge_type: "imports_deferred".into(),
            source_file: "src/b.ts".into(),
            source_symbol: None,
            target_file: "src/a.ts".into(),
            target_symbol: Some("./a".into()),
            line: 1,
            provenance: EdgeProvenance::Explicit,
            via: vec![],
        },
    ];

    let cycles_mixed = detect_import_cycles(&resolved_mixed);
    assert!(
        cycles_mixed.is_empty(),
        "mixed static+dynamic import should NOT form a cycle; got: {:?}",
        cycles_mixed
    );
}
