use arc_swap::ArcSwap;
use dashmap::DashMap;
use petgraph::stable_graph::StableGraph;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::config::ProjectConfig;
use crate::embed::{Embedder, NoopEmbedder, VectorStore};
use crate::search::Bm25Index;
pub use crate::status::{Confidence, PageStatus, Priority};

// ─── Edge Types ─────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum EdgeType {
    Extends,
    Implements,
    ExampleOf,
    PartOf,
    RelatesTo,
    Supports,
    Contradicts,
    Supersedes,
    DependsOn,
    RequiredBy,
    Questions,
    Answers,
    References,
    SimilarTo,
    Causes,
    Mitigates,
    Custom(String),
}

impl EdgeType {
    pub fn priority(&self) -> u8 {
        match self {
            EdgeType::Extends => 10,
            EdgeType::Implements => 9,
            EdgeType::PartOf => 8,
            EdgeType::Supports => 7,
            EdgeType::ExampleOf => 6,
            EdgeType::DependsOn | EdgeType::RequiredBy => 5,
            EdgeType::Mitigates | EdgeType::Causes => 4,
            EdgeType::Contradicts | EdgeType::Questions => 3,
            EdgeType::Answers => 2,
            EdgeType::References | EdgeType::SimilarTo => 1,
            EdgeType::RelatesTo | EdgeType::Custom(_) => 0,
            EdgeType::Supersedes => 8,
        }
    }
}

// ─── Page Types ─────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PageType {
    Task,
    Spec,
    Concept,
    Pattern,
    Decision,
    Howto,
    Reference,
}

// ─── Memory Entry ───────────────────────────────────────────

impl PageType {
    /// Priority rank for sorting (higher = more important)
    pub fn priority_rank(&self) -> u8 {
        match self {
            PageType::Task => 7,
            PageType::Spec => 6,
            PageType::Pattern => 5,
            PageType::Concept => 4,
            PageType::Decision => 3,
            PageType::Howto => 2,
            PageType::Reference => 1,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ─── Spec-specific fields ───────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FunctionalRequirement {
    pub id: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NonFunctionalRequirement {
    pub id: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneralGoal {
    pub description: String,
}

// ─── Decision-specific fields ───────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecisionEntry {
    pub context: String,
    pub options: Vec<String>,
    pub rationale: String,
    pub outcome: String,
}

// ─── Pattern-specific fields ────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatternInfo {
    pub when_to_use: String,
    pub example: String,
}

// ─── Acceptance Criterion ───────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub text: String,
    pub checked: bool,
}

// ─── Wiki Page Metadata ─────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WikiPageMeta {
    pub id: String,
    pub title: String,
    pub page_type: PageType,
    pub tags: Vec<String>,
    pub status: PageStatus,
    pub priority: Option<Priority>,
    pub confidence: Option<Confidence>,
    pub assignee: Option<String>,
    pub aliases: Vec<String>,
    pub superseded_by: Option<String>,
    pub version: Option<String>,
    pub sources: Vec<String>,
    // Per-type structured fields
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub estimate: Option<u32>,
    pub functional_requirements: Vec<FunctionalRequirement>,
    pub non_functional_requirements: Vec<NonFunctionalRequirement>,
    pub general_goals: Vec<GeneralGoal>,
    pub stakeholders: Vec<String>,
    pub decision: Option<DecisionEntry>,
    pub pattern: Option<PatternInfo>,
    pub prerequisites: Vec<String>,
    pub difficulty: Option<String>,
    pub source_url: Option<String>,
    // Relationships as string list
    pub relates_to: Vec<String>,
    // Path & timestamps
    pub path: PathBuf,
    pub created_at: String,
    pub updated_at: String,
}

// ─── Wiki Page Content ──────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WikiPageContent {
    pub raw: String,
    pub sections: Vec<SectionDoc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectionDoc {
    pub section_id: String,
    pub page_id: String,
    pub header: String,
    pub body: String,
}

// ─── Source Entry ───────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SourceState {
    Pending,
    Processing,
    Done,
    Error,
    Stale,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceEntry {
    pub id: String,
    pub original_path: Option<String>,
    pub stored_path: PathBuf,
    pub content_hash: String,
    pub state: SourceState,
    pub added_at: String,
    pub last_processed_at: Option<String>,
    pub page_refs: Vec<String>,
    pub page_count: usize,
    pub error_message: Option<String>,
    pub retry_count: usize,
}

// ─── Audit Event ────────────────────────────────────────────

#[derive(Clone, Debug, Serialize)]
pub struct AuditEvent {
    pub timestamp: String,
    pub tool_name: String,
    pub action: String,
    pub duration_ms: i64,
    pub result: String,
    pub error_message: Option<String>,
    pub entity_refs: Vec<String>,
}

// ─── Write Channel ──────────────────────────────────────────

/// A file write operation for the sequential write channel
#[derive(Debug)]
pub enum WriteOp {
    Write { path: PathBuf, content: Vec<u8> },
    Append { path: PathBuf, content: Vec<u8> },
    Delete { path: PathBuf },
    /// Barrier — sender awaits until consumer processes all prior ops
    Flush { done: tokio::sync::oneshot::Sender<()> },
}

/// Sequential write channel — all disk writes route through this.
/// Ensures concurrent writes don't race.
pub struct WriteChannel {
    sender: tokio::sync::mpsc::UnboundedSender<WriteOp>,
}

impl WriteChannel {
    pub fn new() -> (Self, tokio::sync::mpsc::UnboundedReceiver<WriteOp>) {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        (Self { sender }, receiver)
    }

    pub fn write(&self, path: PathBuf, content: Vec<u8>) -> Result<(), String> {
        self.sender
            .send(WriteOp::Write { path, content })
            .map_err(|_| String::from("channel closed"))
    }

    pub fn append(&self, path: PathBuf, content: Vec<u8>) -> Result<(), String> {
        self.sender
            .send(WriteOp::Append { path, content })
            .map_err(|_| String::from("channel closed"))
    }

    pub fn delete(&self, path: PathBuf) -> Result<(), String> {
        self.sender
            .send(WriteOp::Delete { path })
            .map_err(|_| String::from("channel closed"))
    }

    /// Block until all prior operations have been flushed to disk.
    pub async fn flush(&self) -> Result<(), String> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.sender
            .send(WriteOp::Flush { done: tx })
            .map_err(|_| "channel closed".to_string())?;
        rx.await.map_err(|_| "flush failed".to_string())
    }

    /// Spawn the consumer that processes writes sequentially.
    pub fn spawn_consumer(
        mut receiver: tokio::sync::mpsc::UnboundedReceiver<WriteOp>,
        _project_root: PathBuf,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(op) = receiver.recv().await {
                // All file I/O is blocking — offload to blocking thread pool
                let result = tokio::task::spawn_blocking(move || match op {
                    WriteOp::Write { path, content } => {
                        if let Some(parent) = path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        if let Err(e) = std::fs::write(&path, &content) {
                            tracing::error!("WriteChannel write error {}: {}", path.display(), e);
                        }
                    }
                    WriteOp::Append { path, content } => {
                        if let Some(parent) = path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        if let Ok(mut file) = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&path)
                        {
                            use std::io::Write;
                            if let Err(e) = file.write_all(&content) {
                                tracing::error!(
                                    "WriteChannel append error {}: {}",
                                    path.display(),
                                    e
                                );
                            }
                        }
                    }
                    WriteOp::Delete { path } => {
                        if let Err(e) = std::fs::remove_file(&path) {
                            if e.kind() != std::io::ErrorKind::NotFound {
                                tracing::error!(
                                    "WriteChannel delete error {}: {}",
                                    path.display(),
                                    e
                                );
                            }
                        }
                    }
                    WriteOp::Flush { done } => {
                        // Signal caller that all prior ops are committed to disk
                        let _ = done.send(());
                    }
                })
                .await;
                if let Err(e) = result {
                    tracing::error!("WriteChannel spawn_blocking panicked: {}", e);
                }
            }
        })
    }
}

// ─── Graph Snapshot ─────────────────────────────────────────

pub type GraphSnapshot = (
    petgraph::stable_graph::StableGraph<WikiPageMeta, EdgeType>,
    HashMap<String, petgraph::stable_graph::NodeIndex>,
);

// ─── Engine State ───────────────────────────────────────────

/// Debounced index scheduler — coalesces rapid mutations into single rebuilds.
pub struct IndexScheduler {
    debounce: std::time::Duration,
    cancel_tx: std::sync::Mutex<HashMap<String, tokio::sync::oneshot::Sender<()>>>,
}

impl IndexScheduler {
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            debounce: std::time::Duration::from_millis(debounce_ms),
            cancel_tx: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Submit a rebuild job. If the same `job_type` already has a pending job,
    /// it gets cancelled and replaced (debounce reset).
    pub fn submit<F>(&self, job_type: &str, rebuild_fn: F)
    where
        F: Fn() + Send + 'static,
    {
        // Cancel existing pending job for this type
        match self.cancel_tx.lock() {
            Ok(mut map) => {
                if let Some(tx) = map.remove(job_type) {
                    let _ = tx.send(());
                }
                // Create new cancel channel
                let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();
                map.insert(job_type.to_string(), tx);
                let debounce = self.debounce;
                tokio::spawn(async move {
                    tokio::select! {
                        _ = tokio::time::sleep(debounce) => {
                            rebuild_fn();
                        }
                        _ = &mut rx => {
                            // Cancelled — another submit came in
                        }
                    }
                });
            }
            Err(poisoned) => {
                tracing::error!(
                    "IndexScheduler cancel_tx mutex poisoned for job '{}': {}",
                    job_type,
                    poisoned
                );
            }
        }
    }
}

pub struct EngineState {
    pub graph: ArcSwap<GraphSnapshot>,
    pub page_contents: DashMap<String, WikiPageContent>,
    pub section_corpus: ArcSwap<Vec<SectionDoc>>,
    pub bm25_index: ArcSwap<Bm25Index>,
    pub memory_index: ArcSwap<Bm25Index>,
    pub vector_registry: ArcSwap<HashMap<String, Vec<f32>>>,
    pub source_registry: RwLock<HashMap<String, SourceEntry>>,
    pub content_hashes: RwLock<HashMap<String, String>>,
    pub stale_flag: AtomicBool,
    pub config: RwLock<ProjectConfig>,
    pub audit_sender: tokio::sync::mpsc::Sender<AuditEvent>,
    pub audit_drops: AtomicU64,
    pub started_at: Instant,
    pub project_root: RwLock<PathBuf>,
    // Embedding / vector store
    pub embedder: Box<dyn Embedder + Send + Sync>,
    pub vector_store: VectorStore,
    // Skill system
    pub skill_engine: std::sync::RwLock<crate::skill::SkillEngine>,
    // Two-tier staleness: last-known mtime for external edit detection
    pub wiki_dir_mtime: std::sync::Mutex<Option<std::time::SystemTime>>,
    pub memory_dir_mtime: std::sync::Mutex<Option<std::time::SystemTime>>,
    // Sequential file write channel
    pub write_channel: WriteChannel,
    // Debounced index scheduler
    pub index_scheduler: IndexScheduler,
}

impl EngineState {
    pub fn new(config: ProjectConfig) -> (Self, tokio::sync::mpsc::Receiver<AuditEvent>) {
        let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let (audit_sender, audit_receiver) = tokio::sync::mpsc::channel(1024);
        let (embedder, vector_store) = init_embedder(&config);
        let (write_channel, write_receiver) = WriteChannel::new();
        let _write_handle = WriteChannel::spawn_consumer(write_receiver, project_root.clone());
        let debounce_ms = config.search.scoring.debounce_ms;
        (
            Self {
                graph: ArcSwap::new(Arc::new((
                    StableGraph::<WikiPageMeta, EdgeType>::new(),
                    HashMap::new(),
                ))),
                page_contents: DashMap::new(),
                section_corpus: ArcSwap::new(Arc::new(Vec::new())),
                bm25_index: ArcSwap::new(Arc::new(Bm25Index::new())),
                memory_index: ArcSwap::new(Arc::new(Bm25Index::new())),
                vector_registry: ArcSwap::new(Arc::new(HashMap::new())),
                source_registry: RwLock::new(HashMap::new()),
                content_hashes: RwLock::new(HashMap::new()),
                stale_flag: AtomicBool::new(true),
                config: RwLock::new(config),
                audit_sender,
                audit_drops: AtomicU64::new(0),
                started_at: Instant::now(),
                project_root: RwLock::new(project_root),
                embedder,
                vector_store,
                wiki_dir_mtime: std::sync::Mutex::new(None),
                memory_dir_mtime: std::sync::Mutex::new(None),
                skill_engine: std::sync::RwLock::new(crate::skill::SkillEngine::new()),
                write_channel,
                index_scheduler: IndexScheduler::new(debounce_ms),
            },
            audit_receiver,
        )
    }

    /// Resolve a relative path against the project root
    pub fn resolve_path(&self, relative: &Path) -> PathBuf {
        if let Ok(root) = self.project_root.read() {
            root.join(relative)
        } else {
            relative.to_path_buf()
        }
    }

    /// Check if the wiki directory has been modified externally (git pull, editor save).
    /// Sets stale_flag if mtime changed since last rebuild.
    pub fn check_external_staleness(&self, wiki_dir: &Path) {
        if self.stale_flag.load(Ordering::Acquire) {
            return; // Already stale
        }
        let current_mtime = std::fs::metadata(wiki_dir).and_then(|m| m.modified()).ok();
        match self.wiki_dir_mtime.lock() {
            Ok(stored) => {
                if let (Some(current), Some(prev)) = (current_mtime, *stored) {
                    if current > prev {
                        tracing::debug!("Wiki directory mtime changed — marking stale");
                        self.stale_flag.store(true, Ordering::Release);
                    }
                }
            }
            Err(poisoned) => {
                tracing::error!("wiki_dir_mtime mutex poisoned in check_external_staleness: {}", poisoned);
            }
        }
    }

    /// Rebuild graph snapshot and update mtime tracking.
    pub fn rebuild_graph(&self, wiki_dir: &Path) -> usize {
        let custom_types = match self.config.read() {
            Ok(cfg) => cfg.custom_edge_types.clone(),
            Err(_) => {
                tracing::error!("config lock poisoned in rebuild_graph");
                Vec::new()
            }
        };
        let count = crate::graph::rebuild_snapshot(&self.graph, wiki_dir, &custom_types);
        self.update_wiki_mtime(wiki_dir);
        count
    }

    /// Update the stored wiki directory mtime after rebuild.
    pub fn update_wiki_mtime(&self, wiki_dir: &Path) {
        let mtime = std::fs::metadata(wiki_dir).and_then(|m| m.modified()).ok();
        match self.wiki_dir_mtime.lock() {
            Ok(mut stored) => {
                *stored = mtime;
            }
            Err(poisoned) => {
                tracing::error!("wiki_dir_mtime mutex poisoned in update_wiki_mtime: {}", poisoned);
            }
        }
    }

    /// Rebuild the memory BM25 index from .wm/memory/*.json files.
    /// Delegates BM25 building logic to search::rebuild_memory_index_from_dir.
    pub fn rebuild_memory_index(&self, memory_dir: &Path) -> usize {
        let (index, count) = crate::search::rebuild_memory_index_from_dir(memory_dir);
        self.memory_index.store(Arc::new(index));
        self.update_memory_mtime(memory_dir);
        tracing::info!("Memory index rebuilt: {} entries", count);
        count
    }

    /// Check if memory directory mtime changed.
    pub fn check_memory_staleness(&self, memory_dir: &Path) -> bool {
        let current_mtime = std::fs::metadata(memory_dir).and_then(|m| m.modified()).ok();
        match self.memory_dir_mtime.lock() {
            Ok(stored) => {
                if let (Some(current), Some(prev)) = (current_mtime, *stored) {
                    if current > prev {
                        return true;
                    }
                }
                false
            }
            Err(poisoned) => {
                tracing::error!("memory_dir_mtime mutex poisoned in check_memory_staleness: {}", poisoned);
                false
            }
        }
    }

    /// Update the stored memory directory mtime.
    pub fn update_memory_mtime(&self, memory_dir: &Path) {
        let mtime = std::fs::metadata(memory_dir).and_then(|m| m.modified()).ok();
        match self.memory_dir_mtime.lock() {
            Ok(mut stored) => {
                *stored = mtime;
            }
            Err(poisoned) => {
                tracing::error!("memory_dir_mtime mutex poisoned in update_memory_mtime: {}", poisoned);
            }
        }
    }

    /// Scan skills directory and parse skill files
    pub fn scan_skills(&self, skills_dir: &Path) {
        if let Ok(mut engine) = self.skill_engine.write() {
            engine.scan(skills_dir);
            tracing::info!("Loaded {} skill(s)", engine.list().len());
        }
    }

    /// Fire a skill trigger event
    pub fn fire_skill_event(&self, event: &crate::skill::TriggerEvent) {
        if let Ok(engine) = self.skill_engine.read() {
            engine.fire_event(event, &|tool, action, result, dur, err, refs| {
                self.emit_audit(tool, action, result, dur, err, refs);
            });
        }
    }

    /// Emit an audit event. Uses try_send — drops oldest on overflow, increments drop counter.
    pub fn emit_audit(
        &self,
        tool_name: &str,
        action: &str,
        result: &str,
        duration_ms: i64,
        error_message: Option<String>,
        entity_refs: Vec<String>,
    ) {
        let event = AuditEvent {
            timestamp: chrono::Utc::now().to_rfc3339(),
            tool_name: tool_name.to_string(),
            action: action.to_string(),
            duration_ms,
            result: result.to_string(),
            error_message,
            entity_refs,
        };
        if self.audit_sender.try_send(event).is_err() {
            self.audit_drops.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Initialize embedder and vector store at startup.
/// Tries ONNX first, falls back to NoopEmbedder gracefully.
fn init_embedder(_config: &ProjectConfig) -> (Box<dyn Embedder + Send + Sync>, VectorStore) {
    #[cfg(feature = "embed")]
    {
        let model_name = &_config.embedding.model_name;
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".into());
        let model_cache = PathBuf::from(home).join(".wm").join("models");

        match crate::onnx::OnnxEmbedder::load(&model_cache, model_name) {
            Ok(Some(e)) => {
                tracing::info!(
                    "ONNX embedder loaded: {} ({} dims)",
                    e.model_name(),
                    e.output_dim()
                );
                // Load vectors from project-local state dir, not home
                let project_root = std::env::current_dir().unwrap_or_default();
                let vectors_path = project_root.join(".wm").join("state").join("vectors.bin");
                let vector_store = VectorStore::load_from_disk(&vectors_path)
                    .and_then(|store| {
                        // Validate model name: if mismatch, invalidate vectors
                        if store.model_name == *model_name {
                            Ok(store)
                        } else {
                            tracing::warn!(
                                "vectors.bin was built with '{}' but current model is '{}'. Re-embedding needed.",
                                store.model_name, model_name
                            );
                            Err(format!("model mismatch: {} != {}", store.model_name, model_name))
                        }
                    })
                    .unwrap_or_else(|e| {
                        tracing::warn!("vectors.bin load: {} — starting fresh", e);
                        VectorStore::new(model_name)
                    });
                (Box::new(e) as Box<dyn Embedder + Send + Sync>, vector_store)
            }
            Ok(None) => {
                tracing::info!(
                    "No model found. Run `wm model download {}` for semantic search.",
                    model_name
                );
                (
                    Box::new(NoopEmbedder::new()) as Box<dyn Embedder + Send + Sync>,
                    VectorStore::new(model_name),
                )
            }
            Err(e) => {
                tracing::warn!("ONNX load failed: {} — falling back to BM25-only", e);
                (
                    Box::new(NoopEmbedder::new()) as Box<dyn Embedder + Send + Sync>,
                    VectorStore::new(model_name),
                )
            }
        }
    }

    #[cfg(not(feature = "embed"))]
    {
        tracing::info!("Embedding feature disabled. BM25-only mode.");
        (
            Box::new(NoopEmbedder::new()) as Box<dyn Embedder + Send + Sync>,
            VectorStore::new("none"),
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
        let (state, mut audit_receiver) = EngineState::new(config);
        let state = Arc::new(state);
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        // Spawn audit log consumer
        let handle = tokio::spawn(async move {
            let log_path = std::path::Path::new(".wm").join("audit.jsonl");
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

    pub fn mark_stale(&self) {
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
        let count = crate::graph::rebuild_snapshot(&self.state.graph, wiki_dir, &custom_types);
        self.state.update_wiki_mtime(wiki_dir);
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_type_priority_rank() {
        assert_eq!(PageType::Task.priority_rank(), 7);
        assert_eq!(PageType::Spec.priority_rank(), 6);
        assert_eq!(PageType::Pattern.priority_rank(), 5);
        assert_eq!(PageType::Concept.priority_rank(), 4);
        assert_eq!(PageType::Decision.priority_rank(), 3);
        assert_eq!(PageType::Howto.priority_rank(), 2);
        assert_eq!(PageType::Reference.priority_rank(), 1);
    }

    #[test]
    fn test_edge_type_priority() {
        assert!(EdgeType::Extends.priority() > EdgeType::RelatesTo.priority());
        assert!(EdgeType::Implements.priority() > EdgeType::References.priority());
        assert_eq!(EdgeType::DependsOn.priority(), EdgeType::RequiredBy.priority());
    }

}
