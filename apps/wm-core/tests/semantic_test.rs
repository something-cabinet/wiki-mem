
#![cfg(feature = "onnx")]

#[path = "helpers/mcp_basic.rs"]
mod helpers;
use helpers::MCPClient;

#[path = "helpers/setup.rs"]
mod setup;

fn setup_mcp_test() -> (tempfile::TempDir, MCPClient) {
    let (dir, root) = setup::setup_test_project();
    let client = MCPClient::start(&root);
    (dir, client)
}

fn create_test_page(
    client: &mut MCPClient,
    path: &str,
    title: &str,
    content: &str,
) -> String {
    let created = client
        .call_tool(
            "wm_page",
            serde_json::json!({
                "action": "create",
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


#[test]
fn test_semantic_search_model_available() {
    if std::env::var("TEST_SEMANTIC").unwrap_or_default() != "1" {
        return;
    }

    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

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

    client
        .call_tool("wm_index.rebuild", serde_json::json!({}))
        .expect("index.rebuild failed");

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

    client
        .call_tool("wm_index.rebuild", serde_json::json!({}))
        .expect("index.rebuild failed");

    let status = client
        .call_tool("wm_model", serde_json::json!({ "action": "status" }))
        .expect("model.status failed");

    if !status
        .get("loaded")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
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


#[test]
fn test_semantic_degradation_no_model() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    create_test_page(
        &mut client,
        "concepts/caching",
        "Caching",
        "# Caching\n\nCaching stores frequently accessed data in fast memory to reduce latency.",
    );

    client
        .call_tool(
            "wm_index.rebuild",
            serde_json::json!({ "skip_embed": true }),
        )
        .expect("index rebuild failed");

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
        err.contains("unavailable") || err.contains("model") || err.contains("embeddings") || err.contains("indexed"),
        "semantic search should fail without a model: {}",
        err
    );

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

    let mode = result.get("mode").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        mode == "keyword" || mode == "hybrid",
        "hybrid mode should fall back to keyword or hybrid when no model loaded, got: {}",
        mode
    );
    let embedder_loaded = result
        .get("embedder_loaded")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    // embedder may or may not be loaded depending on whether the model binary exists,
    // but we verify the search still works via BM25 fallback
    let results = result
        .get("results")
        .and_then(|v| v.as_array())
        .unwrap();
    assert!(
        !results.is_empty(),
        "fallback search should return BM25 results"
    );

    for r in results.iter() {
        assert!(r.get("id").and_then(|v| v.as_str()).is_some());
        assert!(r.get("score").and_then(|v| v.as_f64()).is_some());
    }
}


#[test]
fn test_model_status_endpoint() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let result = client
        .call_tool("wm_model", serde_json::json!({ "action": "status" }))
        .expect("model.status failed");

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
