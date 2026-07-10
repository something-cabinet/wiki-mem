//! Engine state — central runtime state for the wiki memory engine.
//! Holds all live data (graph, indexes, config, embedder, audit, skills)
//! and provides methods for mutation, staleness detection, and rebuild.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use arc_swap::ArcSwap;
use dashmap::DashMap;
use petgraph::stable_graph::StableGraph;
use crate::config::ProjectConfig;
use crate::embed::{Embedder, VectorStore};
use crate::search::Bm25Index;
use super::main::init_embedder;
use super::write_channel::WriteChannel;
use super::scheduler::IndexScheduler;
use super::{AuditEvent, EdgeType, GraphSnapshot, SectionDoc, SourceEntry, WikiPageContent, WikiPageMeta};

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
