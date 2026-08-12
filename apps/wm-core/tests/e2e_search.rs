//! Search ranking and resolution contracts through the real `wm-cli` binary.

#[path = "helpers/cli.rs"]
mod helpers;
use helpers::{run_cli, run_cli_with_stdin};

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

#[path = "helpers/macros.rs"]
mod _macros;

#[test]
fn search_query_finds_content() {
    let (_dir, root) = setup_test_project();
    run_cli_with_stdin(
        &root,
        &["page", "create", "concepts/e2e-vector", "Vector Test"],
        "Testing vector search functionality with meaningful content.",
    );
    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    let res = run_cli(&root, &["search", "query", "vector search", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    let results = parsed.get("results").and_then(|v| v.as_array()).expect("results");
    assert!(!results.is_empty(), "expected at least 1 search result");
    assert!(parsed.get("total").and_then(|v| v.as_u64()).unwrap_or(0) >= 1);
}

/// Stemming + rerank must rank a page matching both query terms above a page
/// matching only one (regression: "design pattern" ranked a partial match
/// first).
#[test]
fn search_stemming_and_rerank_ranks_relevant_first() {
    let (_dir, root) = setup_test_project();
    run_cli_with_stdin(
        &root,
        &["page", "create", "reference/my-design-patterns", "Design Patterns Reference"],
        "The 22 classic GoF design patterns, DDD tactical patterns, SOLID principles, and CDD patterns.",
    );
    run_cli_with_stdin(
        &root,
        &["page", "create", "specs/my-design-vocabulary", "Design Vocabulary"],
        "Our design system uses a minimal, neutral aesthetic. Style keywords: accessible, softly rounded.",
    );

    let res = run_cli(&root, &["search", "query", "design pattern", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    let results = parsed["results"].as_array().expect("results array");
    assert!(results.len() >= 2, "expected at least 2 results, got {}", results.len());

    let relevant_pos = results
        .iter()
        .position(|r| r["id"].as_str().is_some_and(|id| id.contains("my-design-patterns")))
        .unwrap_or(usize::MAX);
    let tangential_pos = results
        .iter()
        .position(|r| r["id"].as_str().is_some_and(|id| id.contains("my-design-vocabulary")))
        .unwrap_or(usize::MAX);
    assert!(
        relevant_pos < tangential_pos,
        "design-patterns must rank above design-vocabulary"
    );
}

/// Page IDs with #section anchors must resolve to the canonical page.
#[test]
fn page_get_with_hash_anchor_resolves() {
    let (_dir, root) = setup_test_project();
    run_cli_with_stdin(
        &root,
        &["page", "create", "reference/test-hash", "Hash Test Page"],
        "Testing hash anchor resolution in page IDs.",
    );

    let res = run_cli(&root, &["page", "get", "wiki:reference:test-hash#overview", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    assert!(
        parsed["id"].as_str().is_some_and(|id| id.contains("wiki:reference:test-hash")),
        "#anchor must resolve to test-hash, got: {:?}",
        parsed["id"]
    );

    let res = run_cli(&root, &["page", "get", "wiki:reference:test-hash#overview"]);
    assert_success!(res);
    assert!(res.stdout.contains("Hash Test Page"), "plain output should show the title");
}

/// Plural and singular queries must both match docs containing either form.
#[test]
fn search_plural_and_singular_symmetry() {
    let (_dir, root) = setup_test_project();
    run_cli_with_stdin(
        &root,
        &["page", "create", "reference/symmetry-test", "Patterns Test"],
        "This page discusses various design patterns and their applications.",
    );
    run_cli_with_stdin(
        &root,
        &["page", "create", "concepts/symmetry-singular", "Pattern Example"],
        "A single pattern example for testing.",
    );
    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    for query in ["pattern", "patterns"] {
        let res = run_cli(&root, &["search", "query", query, "--json"]);
        assert_success!(res);
        let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
        let results = parsed["results"].as_array().expect("results array");
        assert!(results.iter().any(|r| r["id"].as_str().is_some_and(|id| id.contains("symmetry-test"))),
            "{query} should find the plural-form page");
        assert!(results.iter().any(|r| r["id"].as_str().is_some_and(|id| id.contains("symmetry-singular"))),
            "{query} should find the singular-form page");
    }
}

#[test]
fn search_retrieve_context() {
    let (_dir, root) = setup_test_project();
    run_cli_with_stdin(
        &root,
        &["page", "create", "tasks/e2e-retrieve-task", "Retrieve Task"],
        "E2E test: This task demonstrates context retrieval.",
    );
    run_cli_with_stdin(
        &root,
        &["page", "create", "specs/e2e-retrieve-spec", "Retrieve Spec"],
        "E2E test spec: FR-2 requires context retrieval.",
    );
    let res = run_cli(
        &root,
        &["search", "retrieve", "E2E test", "--token-budget", "4096", "--json"],
    );
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    assert!(parsed.get("context").is_some(), "retrieve should return context");
    assert!(parsed.get("tokens_used").is_some(), "retrieve should return tokens_used");
}
