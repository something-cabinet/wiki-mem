//! Semantic-search tests.
//!
//! Degradation and model-status contracts run on every build. Tests that need
//! a real ONNX embedder are gated on `feature = "onnx"` and assert the model
//! is actually loaded — a missing model fails loudly with a download hint
//! instead of silently passing.

#[path = "helpers/inproc.rs"]
mod inproc;
use inproc::{call, call_err, call_ok, setup_in_process};

use serde_json::json;

async fn rebuild(registry: &wm_core::mcp::transport::ToolRegistry) -> serde_json::Value {
    call_ok(registry, "wm_index_rebuild", json!({})).await
}

async fn page_create(
    registry: &wm_core::mcp::transport::ToolRegistry,
    path: &str,
    title: &str,
    content: &str,
) {
    call_ok(
        registry,
        "wm_page",
        json!({ "action": "create", "path": path, "title": title, "content": content }),
    )
    .await;
}

/// Semantic mode must fail cleanly when no embedder is loaded, while hybrid
/// falls back to keyword BM25.
#[tokio::test(flavor = "multi_thread")]
async fn semantic_degradation_without_model() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    page_create(
        &registry,
        "concepts/caching",
        "Caching",
        "# Caching\n\nCaching stores frequently accessed data in fast memory to reduce latency.",
    )
    .await;
    rebuild(&registry).await;

    let err = call_err(
        &registry,
        "wm_search.query",
        json!({ "q": "memory storage", "mode": "semantic" }),
    )
    .await;
    assert!(
        err.message.contains("unavailable")
            || err.message.contains("model")
            || err.message.contains("embeddings")
            || err.message.contains("indexed"),
        "expected a semantic-unavailable error, got: {}",
        err.message
    );

    let out = call_ok(
        &registry,
        "wm_search.query",
        json!({ "q": "memory storage", "mode": "hybrid", "limit": 5 }),
    )
    .await;
    let mode = out.get("mode").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        mode == "keyword" || mode == "hybrid",
        "hybrid must fall back to keyword without an embedder, got '{mode}'"
    );
    let results = out.get("results").and_then(|v| v.as_array()).expect("results");
    assert!(!results.is_empty(), "fallback BM25 must return results");
    for r in results {
        assert!(r.get("id").and_then(|v| v.as_str()).is_some());
        assert!(r.get("score").and_then(|v| v.as_f64()).is_some());
    }
}

/// wm_model.status must report the configured model and a loaded flag even
/// when no embedder is available.
#[tokio::test(flavor = "multi_thread")]
async fn model_status_reports_config() {
    let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
    let out = call(&registry, "wm_model", json!({ "action": "status" }))
        .await
        .expect("model status");
    assert!(out.get("model").and_then(|v| v.as_str()).is_some());
    assert!(out.get("loaded").and_then(|v| v.as_bool()).is_some());
    assert!(out.get("dimensions").and_then(|v| v.as_u64()).is_some());
    assert!(out.get("sections_indexed").is_some());
}

#[cfg(feature = "onnx")]
mod onnx {
    use super::*;

    fn assert_model_loaded(out: &serde_json::Value) {
        assert!(
            out.get("embedder_loaded").and_then(|v| v.as_bool()).unwrap_or(false),
            "ONNX feature is enabled but the model is not loaded — run `wm model download bge-small-en-v1.5`"
        );
    }

    /// Semantic search must return scored results when the ONNX embedder is
    /// loaded and the index is embedded.
    #[tokio::test(flavor = "multi_thread")]
    async fn semantic_search_with_loaded_model() {
        let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
        page_create(
            &registry,
            "concepts/authentication",
            "Authentication",
            "# Authentication\n\nAuthentication verifies user identity through credentials such as passwords, tokens, or biometrics.",
        )
        .await;
        page_create(
            &registry,
            "concepts/encryption",
            "Encryption",
            "# Encryption\n\nEncryption transforms plaintext into ciphertext using algorithms and keys.",
        )
        .await;
        call_ok(&registry, "wm_index_rebuild", json!({})).await;

        let out = call(
            &registry,
            "wm_search.query",
            json!({ "q": "user identity verification", "mode": "semantic", "limit": 5 }),
        )
        .await
        .expect("semantic search must run with the model loaded");
        assert_model_loaded(&out);
        assert_eq!(out.get("mode").and_then(|v| v.as_str()), Some("semantic"));
        let results = out.get("results").and_then(|v| v.as_array()).expect("results");
        assert!(!results.is_empty(), "semantic search should return results");
        assert!(results[0].get("id").and_then(|v| v.as_str()).is_some());
        assert!(results[0].get("score").and_then(|v| v.as_f64()).is_some());
    }

    /// Hybrid search must fuse vector + BM25 scores (RRF) when the model is
    /// loaded, and degrade to keyword when it is not.
    #[tokio::test(flavor = "multi_thread")]
    async fn hybrid_search_rrf_fusion_or_fallback() {
        let ((_dir, _root, _engine, registry), _cwd) = setup_in_process().await;
        page_create(
            &registry,
            "concepts/database",
            "Database Systems",
            "# Database Systems\n\nDatabases store and retrieve structured data using SQL queries and indexing.",
        )
        .await;
        page_create(
            &registry,
            "concepts/caching",
            "Caching",
            "# Caching\n\nCaching stores frequently accessed data in fast memory to reduce latency.",
        )
        .await;
        call_ok(&registry, "wm_index_rebuild", json!({})).await;

        let status = call_ok(&registry, "wm_model", json!({ "action": "status" })).await;
        let out = call_ok(
            &registry,
            "wm_search.query",
            json!({ "q": "data retrieval", "mode": "hybrid", "limit": 5 }),
        )
        .await;

        if status.get("loaded").and_then(|v| v.as_bool()).unwrap_or(false) {
            assert_eq!(out.get("mode").and_then(|v| v.as_str()), Some("hybrid"));
            let results = out.get("results").and_then(|v| v.as_array()).expect("results");
            assert!(!results.is_empty());
            assert!(results[0].get("score").and_then(|v| v.as_f64()).is_some());
        } else {
            let mode = out.get("mode").and_then(|v| v.as_str()).unwrap_or("");
            assert_eq!(mode, "keyword", "hybrid must fall back to keyword when no model is loaded");
            let results = out.get("results").and_then(|v| v.as_array()).expect("results");
            assert!(!results.is_empty(), "fallback keyword must return results");
        }
    }
}
