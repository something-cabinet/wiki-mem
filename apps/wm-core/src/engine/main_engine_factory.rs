use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use wm_constants::*;

use super::engine_state_mediator::EngineState;
use crate::config::models::git_tracking_model::{detect_project_root, load_config};
pub use crate::config::ProjectConfig;
use crate::shared::traits::Factory;
use notify_debouncer_full::{
    new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode},
    Debouncer, RecommendedCache,
};
use wm_embed::{Embedder, NoopEmbedder, VectorStore};

pub(super) fn init_embedder(
    _config: &ProjectConfig,
    project_root: &Path,
) -> (Box<dyn Embedder + Send + Sync>, VectorStore) {
    #[cfg(feature = "onnx")]
    {
        let model_name = &_config.embedding.model_name;
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".into());
        let model_cache = PathBuf::from(home).join(WM_DIR).join("models");

        match wm_embed::EmbeddingModel::load(&model_cache, model_name) {
            Ok(Some(e)) => {
                tracing::info!(
                    "ONNX embedder loaded: {} ({} dims)",
                    e.model_name(),
                    e.output_dim()
                );
                let vectors_path = project_root
                    .join(WM_DIR)
                    .join(STATE_DIR)
                    .join(VECTOR_DB_FILE);
                let vector_store = if vectors_path.exists() {
                    VectorStore::load_from_disk(project_root).unwrap_or_else(|e| {
                        tracing::warn!("turso load: {} — starting fresh", e);
                        VectorStore::new(model_name, project_root)
                    })
                } else {
                    let bin_path = project_root
                        .join(WM_DIR)
                        .join(STATE_DIR)
                        .join(VECTOR_BIN_FILE);
                    if bin_path.exists() {
                        match wm_embed::migrate_vectors_bin_to_turso(project_root) {
                            Ok(n) => {
                                tracing::info!("Migrated {} vectors from vectors.bin to turso", n)
                            }
                            Err(e) => tracing::warn!("Migration failed: {}", e),
                        }
                    }
                    VectorStore::new(model_name, project_root)
                };
                let embedder: Box<dyn Embedder + Send + Sync> = Box::new(e);
                (embedder, vector_store)
            }
            Ok(None) => {
                tracing::warn!(
                    "No ONNX model found at {:?}. Semantic search will be unavailable. \
                     Run `wm model download {}` to enable semantic search with BM25+vector hybrid ranking.",
                    model_cache.join(model_name),
                    model_name
                );
                let noop: Box<dyn Embedder + Send + Sync> = Box::new(NoopEmbedder::new());
                (noop, VectorStore::new(model_name, project_root))
            }
            Err(e) => {
                tracing::warn!("ONNX load failed: {} — falling back to BM25-only", e);
                let noop: Box<dyn Embedder + Send + Sync> = Box::new(NoopEmbedder::new());
                (noop, VectorStore::new(model_name, project_root))
            }
        }
    }

    #[cfg(not(feature = "onnx"))]
    {
        tracing::info!("Embedding feature disabled. BM25-only mode.");
        let result: (Box<dyn Embedder + Send + Sync>, VectorStore) = (
            Box::new(NoopEmbedder::new()),
            VectorStore::new("none", project_root),
        );
        result
    }
}

pub struct MainEngine {
    pub state: Arc<EngineState>,
    pub _audit_handle: Option<tokio::task::JoinHandle<()>>,
    pub shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    pub _debouncer: Option<Debouncer<RecommendedWatcher, RecommendedCache>>,
}

impl Factory for MainEngine {}

impl Default for MainEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MainEngine {
    pub fn new() -> Self {
        let project_root = detect_project_root().unwrap_or_else(|| PathBuf::from("."));
        let config = load_config(&project_root).unwrap_or_default();
        Self::with_root(config, project_root)
    }

    pub fn with_root(config: ProjectConfig, project_root: PathBuf) -> Self {
        #[cfg(feature = "code-intel")]
        {
            use crate::code_intel::config_types::LspLanguageSettings as CodeIntelLspSettings;
            use std::collections::HashMap;
            let lsp_converted: Option<HashMap<String, CodeIntelLspSettings>> =
                config.lsp.as_ref().map(|m| {
                    m.iter()
                        .map(|(k, v)| {
                            (
                                k.clone(),
                                CodeIntelLspSettings {
                                    command: v.command.clone(),
                                    args: v.args.clone(),
                                },
                            )
                        })
                        .collect()
                });
            crate::code_intel::load_lsp_config(lsp_converted.as_ref());
        }
        let (state, mut audit_receiver) = EngineState::new(config, project_root.clone());
        let state = Arc::new(state);
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        let log_path = project_root.join(WM_DIR).join(LOG_FILE);
        let handle = tokio::spawn(async move {
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

        let wiki_dir = project_root.join(WM_DIR).join(WIKI_DIR);
        let debouncer = if wiki_dir.is_dir() {
            let (tx, rx) = std::sync::mpsc::channel();
            match new_debouncer(std::time::Duration::from_millis(500), None, tx) {
                Ok(mut debouncer) => {
                    if let Err(e) = debouncer.watch(&wiki_dir, RecursiveMode::Recursive) {
                        tracing::warn!(
                            "File watcher failed to watch {}: {}",
                            wiki_dir.display(),
                            e
                        );
                        None
                    } else {
                        let engine_clone = state.clone();
                        let wd = wiki_dir.clone();
                        std::thread::spawn(move || {
                            for result in rx {
                                match result {
                                    Ok(events) => {
                                        for event in events {
                                            let path: &Path = event.paths[0].as_path();
                                            if path.extension().is_none_or(|e| e != "md") {
                                                continue;
                                            }
                                            let file_name = path
                                                .file_name()
                                                .and_then(std::ffi::OsStr::to_str)
                                                .unwrap_or("");
                                            if file_name == "index.md" || file_name == "log.md" {
                                                continue;
                                            }
                                            use notify_debouncer_full::notify::EventKind;
                                            match event.kind {
                                                EventKind::Create(_) | EventKind::Modify(_) => {
                                                    crate::graph::handle_file_change(
                                                        &wd,
                                                        path,
                                                        &engine_clone,
                                                    );
                                                }
                                                EventKind::Remove(_) => {
                                                    crate::graph::handle_file_delete(
                                                        &wd,
                                                        path,
                                                        &engine_clone,
                                                    );
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    Err(errors) => {
                                        for e in errors {
                                            tracing::warn!("File watcher error: {:?}", e);
                                        }
                                    }
                                }
                            }
                            tracing::info!("File watcher thread exited");
                        });
                        Some(debouncer)
                    }
                }
                Err(e) => {
                    tracing::warn!("File watcher failed to start: {}", e);
                    None
                }
            }
        } else {
            tracing::info!(
                "Wiki directory not found at {}, file watcher disabled.",
                wiki_dir.display()
            );
            None
        };

        Self {
            state,
            _audit_handle: Some(handle),
            shutdown_tx: Some(shutdown_tx),
            _debouncer: debouncer,
        }
    }

    pub fn flag_all_indexes_stale(&self) {
        self.state
            .stale_flag
            .store(true, std::sync::atomic::Ordering::Release);
    }

    pub fn set_project_root(&self, root: PathBuf) {
        if let Ok(mut project_root) = self.state.project_root.write() {
            *project_root = root;
        }
    }

    pub async fn shutdown(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self._audit_handle.take() {
            let _ = handle.await;
        }
    }

    pub fn rebuild_wiki(&self, wiki_dir: &Path) -> usize {
        let custom_types = match self.state.config.read() {
            Ok(cfg) => cfg.custom_edge_types.clone(),
            Err(_) => {
                tracing::error!("config lock poisoned in rebuild_wiki");
                Vec::new()
            }
        };
        let count =
            crate::graph::rebuild_graph_snapshot(&self.state.graph, wiki_dir, &custom_types);
        self.state.update_wiki_mtime(wiki_dir);
        self.state
            .stale_flag
            .store(false, std::sync::atomic::Ordering::Release);
        count
    }
}
