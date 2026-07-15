// ─── Semantic Search E2E Tests ───────────────────────────────
// Feature-gated: #[cfg(feature = "embed")]
// Gated behind `embed` feature since ONNX Runtime is required.
// Tests requiring an actual model need TEST_SEMANTIC=1.
// Model-absence tests use the default NoopEmbedder for graceful degradation.

#![cfg(feature = "embed")]

mod helpers;

use helpers::MCPClient;

/// Create an MCP client connected to a test project.
fn setup_mcp_test() -> (tempfile::TempDir, MCPClient) {
    let (dir, root) = helpers::setup_test_project();
    let client = MCPClient::start(&root);
    (dir, client)
}

/// Create a wiki page and return its ID.
fn create_test_page(
    client: &mut MCPClient,
    path: &str,
    title: &str,
    content: &str,
) -> String {
    let created = client
        .call_tool(
            "wm_page.create",
            serde_json::json!({
                "path": path,
                "title": title,
                "content": content,
            }),
        )
        .expect("page.create failed");
    created
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

// ─── Semantic Search (model required) ────────────────────────

#[test]
fn test_semantic_search_model_available() {
    if std::env::var("TEST_SEMANTIC").unwrap_or_default() != "1" {
        return;
    }

    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    // Create pages with meaningful content for semantic matching
    create_test_page(
        &mut client,
        "concepts/authentication",
        "Authentication",
        "# Authentication\n\nAuthentication verifies user identity through credentials such as passwords, tokens, or biometrics.\n\n## Methods\nCommon methods include password-based, OAuth, SSO, and multi-factor authentication.",
    );
    create_test_page(
        &mut client,
        "concepts/authorization",
        "Authorization",
        "# Authorization\n\nAuthorization controls access to system resources based on roles and permissions.\n\n## RBAC\nRole-based access control assigns permissions to roles rather than individuals.",
    );
    create_test_page(
        &mut client,
        "concepts/encryption",
        "Encryption",
        "# Encryption\n\nEncryption transforms plaintext into ciphertext using algorithms and keys.\n\n## Symmetric vs Asymmetric\nSymmetric encryption uses one key; asymmetric uses a key pair.",
    );

    // Full rebuild (graph + BM25 + embeddings)
    client
        .call_tool("wm_index", serde_json::json!({ "action": "Rebuild" }))
        .expect("index.rebuild failed");

    // Semantic search for authentication-related content
    let result = client
        .call_tool(
            "wm_search.query",
            serde_json::json!({
                "q": "user identity verification",
                "mode": "semantic",
                "limit": 5,
            }),
        )
        .expect("search.query failed");

    assert_eq!(
        result.get("mode").and_then(|v| v.as_str()),
        Some("semantic")
    );
    assert!(
        result
            .get("embedder_loaded")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        "model should be loaded when TEST_SEMANTIC=1"
    );

    let results = result
        .get("results")
        .and_then(|v| v.as_array())
        .unwrap();
    assert!(
        !results.is_empty(),
        "semantic search should return results when model loaded"
    );
    assert!(
        results[0]
            .get("id")
            .and_then(|v| v.as_str())
            .is_some(),
        "result should have an id"
    );
    assert!(
        results[0]
            .get("score")
            .and_then(|v| v.as_f64())
            .is_some(),
        "result should have a score"
    );
}

// ─── Hybrid Search RRF Fusion (model required) ───────────────

#[test]
fn test_hybrid_search_rrf_fusion() {
    if std::env::var("TEST_SEMANTIC").unwrap_or_default() != "1" {
        return;
    }

    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    create_test_page(
        &mut client,
        "concepts/database",
        "Database Systems",
        "# Database Systems\n\nDatabases store and retrieve structured data using SQL queries and indexing.\n\n## Transactions\nACID transactions ensure data consistency in database operations.",
    );
    create_test_page(
        &mut client,
        "concepts/caching",
        "Caching",
        "# Caching\n\nCaching stores frequently accessed data in fast memory (RAM) to reduce latency.\n\n## Cache Strategies\nCommon strategies include LRU, TTL-based, and write-through caching.",
    );

    // Full rebuild (graph + BM25 + embeddings)
    client
        .call_tool("wm_index", serde_json::json!({ "action": "Rebuild" }))
        .expect("index.rebuild failed");

    // Check if model is actually loaded (user may have set TEST_SEMANTIC=1
    // but not downloaded a model — in that case verify graceful fallback).
    let status = client
        .call_tool("wm_model", serde_json::json!({ "action": "Status" }))
        .expect("model.status failed");

    if !status
        .get("loaded")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        // No model loaded despite TEST_SEMANTIC — verify graceful fallback
        let result = client
            .call_tool(
                "wm_search.query",
                serde_json::json!({
                    "q": "data storage",
                    "mode": "hybrid",
                    "limit": 5,
                }),
            )
            .expect("hybrid search should fall back gracefully");

        assert_eq!(
            result.get("mode").and_then(|v| v.as_str()),
            Some("keyword"),
            "hybrid should fall back to keyword when no model loaded"
        );
        let results = result
            .get("results")
            .and_then(|v| v.as_array())
            .unwrap();
        assert!(!results.is_empty(), "fallback keyword should return results");
        return;
    }

    // Full hybrid search with RRF fusion
    let result = client
        .call_tool(
            "wm_search.query",
            serde_json::json!({
                "q": "data retrieval",
                "mode": "hybrid",
                "limit": 5,
            }),
        )
        .expect("hybrid search failed");

    assert_eq!(result.get("mode").and_then(|v| v.as_str()), Some("hybrid"));

    let results = result
        .get("results")
        .and_then(|v| v.as_array())
        .unwrap();
    assert!(
        !results.is_empty(),
        "hybrid search should return results"
    );
    assert!(
        results[0]
            .get("score")
            .and_then(|v| v.as_f64())
            .is_some(),
        "first result should have a fused RRF score"
    );
    assert_eq!(
        result.get("query").and_then(|v| v.as_str()),
        Some("data retrieval")
    );
}

// ─── Graceful Degradation (no model needed) ──────────────────

#[test]
fn test_semantic_degradation_no_model() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    // Create a page so BM25 search has content
    create_test_page(
        &mut client,
        "concepts/caching",
        "Caching",
        "# Caching\n\nCaching stores frequently accessed data in fast memory to reduce latency.",
    );

    // Rebuild with skip_embed to avoid any model interaction
    client
        .call_tool(
            "wm_index",
            serde_json::json!({ "action": "Rebuild", "skip_embed": true }),
        )
        .expect("index rebuild failed");

    // Semantic search should error when no model is loaded
    let err = client
        .call_tool(
            "wm_search.query",
            serde_json::json!({
                "q": "memory storage",
                "mode": "semantic",
            }),
        )
        .unwrap_err();
    assert!(
        err.contains("unavailable") || err.contains("model"),
        "semantic search should fail without a model: {}",
        err
    );

    // Hybrid search should gracefully fall back to keyword
    let result = client
        .call_tool(
            "wm_search.query",
            serde_json::json!({
                "q": "memory storage",
                "mode": "hybrid",
                "limit": 5,
            }),
        )
        .expect("hybrid search should fall back to keyword");

    assert_eq!(
        result.get("mode").and_then(|v| v.as_str()),
        Some("keyword"),
        "hybrid mode should fall back to keyword when no model loaded"
    );
    assert!(
        !result
            .get("embedder_loaded")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        "embedder should not be loaded"
    );

    let results = result
        .get("results")
        .and_then(|v| v.as_array())
        .unwrap();
    assert!(
        !results.is_empty(),
        "fallback keyword search should return BM25 results"
    );

    // Each result should have the expected fields
    for r in results.iter() {
        assert!(r.get("id").and_then(|v| v.as_str()).is_some());
        assert!(r.get("score").and_then(|v| v.as_f64()).is_some());
    }
}

// ─── Model Status Endpoint (no model needed) ─────────────────

#[test]
fn test_model_status_endpoint() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool("wm_model", serde_json::json!({ "action": "Status" }))
        .expect("model.status failed");

    // All fields should be present regardless of model state
    assert!(
        result.get("model").and_then(|v| v.as_str()).is_some(),
        "model.status should return model name"
    );
    assert!(
        result.get("loaded").and_then(|v| v.as_bool()).is_some(),
        "model.status should return loaded flag"
    );
    assert!(
        result.get("dimensions").and_then(|v| v.as_u64()).is_some(),
        "model.status should return dimensions"
    );
    assert!(
        result
            .get("sections_indexed")
            .and_then(|v| v.as_u64())
            .is_some(),
        "model.status should return sections_indexed"
    );
}
