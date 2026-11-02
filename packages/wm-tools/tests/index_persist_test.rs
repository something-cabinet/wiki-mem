#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use serde_json::json;
use wm_core::config::ProjectConfig;
use wm_core::engine::EngineState;
use wm_embed::vector_db::{EmbedError, EmbedVector, MockEmbedder};
use wm_embed::{Embedder, VectorStore};
use wm_tools::tools;
use wm_tools::transport::ToolRegistry;

const EMBED_DIM: usize = 8;
const MODEL_NAME: &str = "mock";
const FIXTURE_PAGE_REL: &str = ".wm/wiki/concepts/embed-fixture.md";
const FIXTURE_PAGE_BODY: &str = "---\ntitle: Embed Fixture\ntype: concept\nstatus: draft\n---\n\n## Overview\n\nA page that gives the index something to embed.\n";
const UNOPENABLE_STATE_DIR_ENTRY: &str = "not-a-directory";

struct CountingEmbedder {
    inner: MockEmbedder,
    calls: Arc<AtomicUsize>,
}

impl CountingEmbedder {
    fn new(calls: Arc<AtomicUsize>) -> Self {
        Self {
            inner: MockEmbedder::new(EMBED_DIM),
            calls,
        }
    }
}

impl Embedder for CountingEmbedder {
    fn embed(&self, text: &str) -> Result<EmbedVector, EmbedError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.inner.embed(text)
    }

    fn is_loaded(&self) -> bool {
        true
    }

    fn model_name(&self) -> &str {
        MODEL_NAME
    }

    fn output_dim(&self) -> usize {
        EMBED_DIM
    }
}

struct Harness {
    _dir: tempfile::TempDir,
    engine: Arc<EngineState>,
    registry: ToolRegistry,
    calls: Arc<AtomicUsize>,
}

fn seed_fixture_page(root: &Path) {
    let page = root.join(FIXTURE_PAGE_REL);
    std::fs::create_dir_all(page.parent().expect("page parent")).expect("create concepts dir");
    std::fs::write(page, FIXTURE_PAGE_BODY).expect("write fixture page");
}

fn harness_with_store(store_root: impl FnOnce(&Path) -> std::path::PathBuf) -> Harness {
    let (dir, root) = setup_test_project();
    seed_fixture_page(&root);
    let calls = Arc::new(AtomicUsize::new(0));
    let store = VectorStore::new(MODEL_NAME, &store_root(&root));
    let (state, _audit) = EngineState::with_embedder(
        ProjectConfig::default(),
        root,
        Box::new(CountingEmbedder::new(calls.clone())),
        store,
    );
    let engine = Arc::new(state);
    let mut registry = ToolRegistry::new();
    tools::register_all_tools(&mut registry, engine.clone());
    Harness {
        _dir: dir,
        engine,
        registry,
        calls,
    }
}

fn persistable_harness() -> Harness {
    harness_with_store(|root| root.to_path_buf())
}

fn unpersistable_harness() -> Harness {
    harness_with_store(|root| {
        let blocker = root.join(UNOPENABLE_STATE_DIR_ENTRY);
        std::fs::write(&blocker, "x").expect("write blocker");
        blocker
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn successful_rebuild_writes_vectors_that_read_back_from_the_database() {
    let harness = persistable_harness();

    let out = harness
        .registry
        .dispatch_async("wm_index_rebuild", json!({}))
        .await
        .expect("rebuild must succeed");

    let embedded = out["sections_embedded"].as_u64().unwrap_or(0) as usize;
    assert!(embedded > 0, "{out}");
    let db = harness
        .engine
        .index
        .vector_store()
        .db
        .as_ref()
        .expect("store must have a database");
    let (persisted, _hashes) = db.load_all_raw().expect("read vectors back");
    assert_eq!(persisted.len(), embedded);
    assert!(harness.calls.load(Ordering::SeqCst) > 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn rebuild_with_an_unpersistable_store_fails_before_embedding() {
    let harness = unpersistable_harness();

    let error = harness
        .registry
        .dispatch_async("wm_index_rebuild", json!({}))
        .await
        .expect_err("rebuild must fail");

    assert!(
        error.message.contains("Cannot persist embeddings"),
        "{}",
        error.message
    );
    assert_eq!(harness.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn embed_with_an_unpersistable_store_fails_before_embedding() {
    let harness = unpersistable_harness();
    harness
        .registry
        .dispatch_async("wm_index_rebuild", json!({ "skip_embed": true }))
        .await
        .expect("a rebuild without embedding must succeed");

    let error = harness
        .registry
        .dispatch_async("wm_index_embed", json!({}))
        .await
        .expect_err("embed must fail");

    assert!(
        error.message.contains("Cannot persist embeddings"),
        "{}",
        error.message
    );
    assert_eq!(harness.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn rebuild_without_embedding_succeeds_even_when_the_store_cannot_persist() {
    let harness = unpersistable_harness();

    let out = harness
        .registry
        .dispatch_async("wm_index_rebuild", json!({ "skip_embed": true }))
        .await
        .expect("a rebuild without embedding must succeed");

    assert_eq!(out["sections_embedded"].as_u64(), Some(0));
    assert_eq!(harness.calls.load(Ordering::SeqCst), 0);
}
