use std::path::Path;
use std::sync::Arc;

use wm_core::engine::EngineState;

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

fn engine_for(root: &Path) -> Arc<EngineState> {
    let config = wm_core::config::ProjectConfig::default();
    let (state, _audit_rx) = EngineState::new(config, root.to_path_buf());
    Arc::new(state)
}

fn write_legacy(root: &Path, file: &str, json: &str) {
    std::fs::write(root.join(".wm/memory").join(file), json).expect("write legacy memory");
}

fn legacy_entry(id: &str, title: &str) -> String {
    serde_json::json!({
        "id": id,
        "title": title,
        "content": "Remember this.",
        "tags": ["alpha"],
        "created_at": "2026-07-01T00:00:00Z",
        "updated_at": "2026-07-01T00:00:00Z"
    })
    .to_string()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_verified_import_removes_the_json_and_counts_the_page() {
    let (_dir, root) = setup_test_project();
    write_legacy(&root, "one.json", &legacy_entry("one", "Entry One"));
    let engine = engine_for(&root);

    let report = wm_core::page::import_legacy_memory(&engine).expect("import");

    assert_eq!(report.imported, 1);
    assert!(report.failed.is_empty());
    assert!(!root.join(".wm/memory/one.json").exists());
    let page = std::fs::read_to_string(root.join(".wm/wiki/memory/one.md")).expect("page");
    assert!(page.contains("Entry One"));
}

#[tokio::test(flavor = "multi_thread")]
async fn an_unreadable_entry_is_kept_and_reported() {
    let (_dir, root) = setup_test_project();
    write_legacy(&root, "bad.json", "not valid json");
    write_legacy(&root, "two.json", &legacy_entry("two", "Entry Two"));
    let engine = engine_for(&root);

    let report = wm_core::page::import_legacy_memory(&engine).expect("import");

    assert_eq!(report.imported, 1);
    assert_eq!(report.failed.len(), 1);
    assert_eq!(report.failed[0].file, "bad.json");
    assert!(root.join(".wm/memory/bad.json").exists());
    assert!(!root.join(".wm/memory/two.json").exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn an_entry_whose_page_cannot_be_written_keeps_its_json() {
    let (_dir, root) = setup_test_project();
    write_legacy(
        &root,
        "evil.json",
        &legacy_entry("../../escape", "Escaping"),
    );
    let engine = engine_for(&root);

    let report = wm_core::page::import_legacy_memory(&engine).expect("import");

    assert_eq!(report.imported, 0);
    assert_eq!(report.failed.len(), 1);
    assert!(root.join(".wm/memory/evil.json").exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn the_legacy_directory_stays_while_a_failure_remains() {
    let (_dir, root) = setup_test_project();
    write_legacy(&root, "bad.json", "{");
    let engine = engine_for(&root);

    let report = wm_core::page::import_legacy_memory(&engine).expect("import");

    assert_eq!(report.failed.len(), 1);
    assert!(root.join(".wm/memory").exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn the_legacy_directory_is_removed_once_everything_is_imported() {
    let (_dir, root) = setup_test_project();
    write_legacy(&root, "one.json", &legacy_entry("one", "Entry One"));
    let engine = engine_for(&root);

    wm_core::page::import_legacy_memory(&engine).expect("import");

    assert!(!root.join(".wm/memory").exists());
}
