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

    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "concepts/e2e-vector", "Vector Test"],
        "Testing vector search functionality with meaningful content for embedding.",
    );
    assert_success!(res);

    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    let state_dir = root.join(".wm").join("state");
    assert!(state_dir.exists(), "state directory should exist");

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

    let res = run_cli(&root, &["search", "query", "E2E Memory", "--json"]);
    assert_success!(res);
}

/// Test that stemming + rerank boost correctly ranks a relevant page
/// higher than a page that only matches partial query terms.
///
/// Reported bug: searching "design pattern" returned
/// `designer-review-followup#style-keywords` (matches only "design") above
/// `reference:design-patterns` (should match both "design" and "pattern").
#[test]
fn search_stemming_and_rerank_ranks_relevant_first() {
    let (_dir, root) = setup_test_project();

    // Create a highly-relevant page: title contains both "design" and "patterns"
    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "reference/my-design-patterns", "Design Patterns Reference"],
        "The 22 classic GoF design patterns, DDD tactical patterns, SOLID principles, and CDD patterns.",
    );
    assert_success!(res);

    // Create a tangentially-related page: title and body mention "design" but NOT "pattern"
    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "specs/my-design-vocabulary", "Design Vocabulary"],
        "Our design system uses a minimal, neutral aesthetic. Style keywords: accessible, softly rounded.",
    );
    assert_success!(res);

    // Search with singular query — stemming should match "patterns" via "pattern"
    let res = run_cli(&root, &["search", "query", "design pattern", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value =
        serde_json::from_str(&res.stdout).expect("valid JSON from search");
    let results = parsed["results"].as_array().expect("results array");
    assert!(
        results.len() >= 2,
        "expected at least 2 results, got {}",
        results.len(),
    );

    // The relevant page (matching both "design" and "pattern") must rank above
    // the tangential page (matching only "design").
    let relevant_pos = results
        .iter()
        .position(|r| {
            r["id"]
                .as_str()
                .map_or(false, |id| id.contains("my-design-patterns"))
        })
        .unwrap_or(usize::MAX);
    let tangential_pos = results
        .iter()
        .position(|r| {
            r["id"]
                .as_str()
                .map_or(false, |id| id.contains("my-design-vocabulary"))
        })
        .unwrap_or(usize::MAX);

    assert!(
        relevant_pos < tangential_pos,
        "design-patterns page should rank above design-vocabulary page.\n\
         scores: patterns={} vocabulary={}",
        results[relevant_pos]["score"],
        results[tangential_pos]["score"],
    );
}

/// Test that page IDs with #section anchors resolve correctly.
/// e.g. "wiki:reference:design-patterns#overview" → "wiki:reference:design-patterns"
#[test]
fn page_get_with_hash_anchor_resolves() {
    let (_dir, root) = setup_test_project();

    // Create a page
    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "reference/test-hash", "Hash Test Page"],
        "Testing hash anchor resolution in page IDs.",
    );
    assert_success!(res);

    // Get with bare ID
    let res = run_cli(
        &root,
        &["page", "get", "wiki:reference:test-hash", "--json"],
    );
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    assert_eq!(
        parsed["id"], "wiki:reference:test-hash",
        "bare ID should work"
    );

    // Get with #section suffix — should resolve to the same page
    let res = run_cli(
        &root,
        &["page", "get", "wiki:reference:test-hash#overview", "--json"],
    );
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    // The page_id in the response should be the canonical page, not including the # anchor
    assert!(
        parsed["id"]
            .as_str()
            .map_or(false, |id| id.contains("wiki:reference:test-hash")),
        "#anchor should resolve to test-hash page, got: {:?}",
        parsed["id"],
    );
    // Stream output (non-JSON) should also work
    let res_plain = run_cli(&root, &["page", "get", "wiki:reference:test-hash#overview"]);
    assert_success!(res_plain);
    assert!(
        res_plain.stdout.contains("Hash Test Page"),
        "should show the page title"
    );
}

/// Test plural/singular search symmetry via CLI e2e
#[test]
fn search_plural_and_singular_symmetry() {
    let (_dir, root) = setup_test_project();

    // Create a page with "patterns" (plural)
    let res = run_cli_with_stdin(
        &root,
        &["page", "create", "reference/symmetry-test", "Patterns Test"],
        "This page discusses various design patterns and their applications.",
    );
    assert_success!(res);

    // Create another page with "pattern" (singular)
    let res = run_cli_with_stdin(
        &root,
        &[
            "page",
            "create",
            "concepts/symmetry-singular",
            "Pattern Example",
        ],
        "A single pattern example for testing.",
    );
    assert_success!(res);

    let res = run_cli(&root, &["index", "rebuild"]);
    assert_success!(res);

    // Search with singular "pattern"
    let res = run_cli(&root, &["search", "query", "pattern", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    let results = parsed["results"].as_array().expect("results array");
    assert!(
        results.iter().any(|r| r["id"]
            .as_str()
            .map_or(false, |id| id.contains("symmetry-test"))),
        "singular query 'pattern' should find doc with 'patterns'"
    );
    assert!(
        results.iter().any(|r| r["id"]
            .as_str()
            .map_or(false, |id| id.contains("symmetry-singular"))),
        "singular query 'pattern' should find doc with 'pattern'"
    );

    // Search with plural "patterns"
    let res = run_cli(&root, &["search", "query", "patterns", "--json"]);
    assert_success!(res);
    let parsed: serde_json::Value = serde_json::from_str(&res.stdout).expect("valid JSON");
    let results = parsed["results"].as_array().expect("results array");
    assert!(
        results.iter().any(|r| r["id"]
            .as_str()
            .map_or(false, |id| id.contains("symmetry-test"))),
        "plural query 'patterns' should find doc with 'patterns'"
    );
    assert!(
        results.iter().any(|r| r["id"]
            .as_str()
            .map_or(false, |id| id.contains("symmetry-singular"))),
        "plural query 'patterns' should find doc with 'pattern'"
    );
}

#[test]
fn search_retrieve_context() {
    let (_dir, root) = setup_test_project();

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

    let res = run_cli(
        &root,
        &[
            "search",
            "retrieve",
            "E2E test",
            "--token-budget",
            "4096",
            "--json",
        ],
    );
    assert_success!(res);
}
