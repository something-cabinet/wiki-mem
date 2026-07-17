//! Main engine bootstrap — initializes [`EngineState`], spawns the audit log
//! consumer, and manages graceful shutdown.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::config::ProjectConfig;
use wm_embed::{Embedder, NoopEmbedder, VectorStore};
use super::state::EngineState;

/// Initialize embedder and vector store at startup.
/// Tries ONNX first, falls back to NoopEmbedder gracefully.
pub(super) fn init_embedder(_config: &ProjectConfig, project_root: &Path) -> (Box<dyn Embedder + Send + Sync>, VectorStore) {
    #[cfg(feature = "embed")]
    {
        let model_name = &_config.embedding.model_name;
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".into());
        let model_cache = PathBuf::from(home).join(".wm").join("models");

        match wm_embed::OnnxEmbedder::load(&model_cache, model_name) {
            Ok(Some(e)) => {
                tracing::info!(
                    "ONNX embedder loaded: {} ({} dims)",
                    e.model_name(),
                    e.output_dim()
                );
                // Load vectors from turso at project-local state dir
                let vectors_path = project_root.join(".wm").join("state").join("vectors.db");
                let vector_store = if vectors_path.exists() {
                    VectorStore::load_from_disk(project_root)
                        .unwrap_or_else(|e| {
                            tracing::warn!("turso load: {} — starting fresh", e);
                            VectorStore::new(model_name, project_root)
                        })
                } else {
                    // Try migrating old vectors.bin
                    let bin_path = project_root.join(".wm").join("state").join("vectors.bin");
                    if bin_path.exists() {
                        match wm_embed::migrate_vectors_bin_to_turso(project_root) {
                            Ok(n) => tracing::info!("Migrated {} vectors from vectors.bin to turso", n),
                            Err(e) => tracing::warn!("Migration failed: {}", e),
                        }
                    }
                    VectorStore::new(model_name, project_root)
                };
                (Box::new(e) as Box<dyn Embedder + Send + Sync>, vector_store)
            }
            Ok(None) => {
                tracing::info!(
                    "No model found. Run `wm model download {}` for semantic search.",
                    model_name
                );
                (
                    Box::new(NoopEmbedder::new()) as Box<dyn Embedder + Send + Sync>,
                    VectorStore::new(model_name, project_root),
                )
            }
            Err(e) => {
                tracing::warn!("ONNX load failed: {} — falling back to BM25-only", e);
                (
                    Box::new(NoopEmbedder::new()) as Box<dyn Embedder + Send + Sync>,
                    VectorStore::new(model_name, project_root),
                )
            }
        }
    }

    #[cfg(not(feature = "embed"))]
    {
        tracing::info!("Embedding feature disabled. BM25-only mode.");
        (
            Box::new(NoopEmbedder::new()) as Box<dyn Embedder + Send + Sync>,
            VectorStore::new("none", project_root),
        )
    }
}

// ─── Wiki Memory Engine ─────────────────────────────────────────

pub struct MainEngine {
    pub state: Arc<EngineState>,
    pub _audit_handle: Option<tokio::task::JoinHandle<()>>,
    pub shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl MainEngine {
    pub fn new(config: ProjectConfig) -> Self {
        #[cfg(feature = "code-intel")]
        crate::code_intel::load_lsp_config(&config);
        let (state, mut audit_receiver) = EngineState::new(config);
        let state = Arc::new(state);
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        // Spawn audit log consumer
        let handle = tokio::spawn(async move {
            let log_path = std::path::Path::new(".wm").join("log.jsonl");
            if let Some(parent) = log_path.parent() {
                let parent = parent.to_path_buf();
                let _ = tokio::task::spawn_blocking(move || {
                    std::fs::create_dir_all(&parent).ok();
                })
                .await;
            }

            loop {
                tokio::select! {
                    event = audit_receiver.recv() => {
                        match event {
                            Some(event) => {
                                if event.tool_name == "help" || event.tool_name == "initial" {
                                    continue;
                                }
                                let log_path = log_path.clone();
                                let _ = tokio::task::spawn_blocking(move || {
                                    if let Ok(line) = serde_json::to_string(&event) {
                                        if let Ok(mut file) = std::fs::OpenOptions::new()
                                            .create(true).append(true).open(&log_path)
                                        {
                                            use std::io::Write;
                                            let _ = writeln!(file, "{}", line);
                                            let _ = file.flush();
                                        }
                                    }
                                })
                                .await;
                            }
                            None => break,
                        }
                    }
                    _ = &mut shutdown_rx => {
                        // Drain remaining events before exit
                        while let Ok(Some(event)) = tokio::time::timeout(
                            std::time::Duration::from_millis(100),
                            audit_receiver.recv(),
                        ).await {
                            let log_path = log_path.clone();
                            let _ = tokio::task::spawn_blocking(move || {
                                if let Ok(line) = serde_json::to_string(&event) {
                                    if let Ok(mut file) = std::fs::OpenOptions::new()
                                        .create(true).append(true).open(&log_path)
                                    {
                                        let _ = writeln!(file, "{}", line);
                                    }
                                }
                            })
                            .await;
                        }
                        break;
                    }
                }
            }
        });

        Self {
            state,
            _audit_handle: Some(handle),
            shutdown_tx: Some(shutdown_tx),
        }
    }

    pub fn flag_all_indexes_stale(&self) {
        self.state
            .stale_flag
            .store(true, std::sync::atomic::Ordering::Release);
    }

    /// Override the project root (used by serve command with --project flag)
    pub fn set_project_root(&self, root: PathBuf) {
        if let Ok(mut project_root) = self.state.project_root.write() {
            *project_root = root;
        }
    }

    /// Signal audit consumer to flush and exit.
    pub async fn shutdown(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self._audit_handle.take() {
            let _ = handle.await;
        }
    }

    /// Rebuild the graph snapshot and update wiki mtime tracking.
    pub fn rebuild_wiki(&self, wiki_dir: &Path) -> usize {
        let custom_types = match self.state.config.read() {
            Ok(cfg) => cfg.custom_edge_types.clone(),
            Err(_) => {
                tracing::error!("config lock poisoned in rebuild_wiki");
                Vec::new()
            }
        };
        let count = crate::graph::rebuild_graph_snapshot(&self.state.graph, wiki_dir, &custom_types);
        self.state.update_wiki_mtime(wiki_dir);
        self.state.stale_flag.store(false, std::sync::atomic::Ordering::Release);
        count
    }
}
