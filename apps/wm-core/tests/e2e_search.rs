// ─── E2E: Search Operations ────────────────────────────────────
// Tests search query and retrieve functionality, including vector
// storage (BM25 fallback) and hybrid search.

mod helpers;

use helpers::{run_cli, run_cli_with_stdin, setup_test_project};

#[test]
fn search_query_finds_content() {
    let (_dir, root) = setup_test_project();

    // Create a page with content that can be indexed
    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "concepts/e2e-vector", "Vector Test"],
        "Testing vector search functionality with meaningful content for embedding.",
    );
    assert_success!(res);

    // Rebuild index (BM25 fallback; embedding skipped if no model loaded)
    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    // State directory should exist
    let state_dir = root.join(".wm").join("state");
    assert!(state_dir.exists(), "state directory should exist");

    // Search for a keyword — hybrid search returns results via BM25
    let res = run_cli(&root, &["search", "query", "vector search", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON from search");
    let results = parsed.get("results").and_then(|v| v.as_array());
    let total = parsed.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    if let Some(r) = results {
        assert!(!r.is_empty(), "expected at least 1 search result, got 0");
    }
    assert!(total >= 1, "expected total >= 1, got {}", total);

    // Also verify the memory page search works
    let res = run_cli(&root, &["search", "query", "E2E Memory", "--json"]);
    assert_success!(res);
}

#[test]
fn search_retrieve_context() {
    let (_dir, root) = setup_test_project();

    // Create pages via stdin
    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/e2e-retrieve-task", "Retrieve Task"],
        "E2E test: This task demonstrates context retrieval.",
    );
    assert_success!(res);

    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "specs/e2e-retrieve-spec", "Retrieve Spec"],
        "E2E test spec: FR-2 requires context retrieval.",
    );
    assert_success!(res);

    // Retrieve context for a query
    let res = run_cli(&root, &[
        "search", "retrieve", "E2E test",
        "--token-budget", "4096", "--json",
    ]);
    assert_success!(res);
}
