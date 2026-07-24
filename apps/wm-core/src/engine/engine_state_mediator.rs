
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use arc_swap::ArcSwap;
use dashmap::DashMap;
use petgraph::stable_graph::StableGraph;
use crate::config::ProjectConfig;
use rmcp::model::Tool;
use wm_embed::{Embedder, VectorStore};
use crate::search::Bm25Index;
use super::main_engine_factory::init_embedder;
use super::write_channel_proxy::WriteChannel;
use super::index_scheduler_service::IndexScheduler;
use super::{AuditEvent, EdgeType, GraphSnapshot, MemoryEntry, SectionDoc, SourceEntry, WikiPageContent, WikiPageMeta};

pub struct EngineState {
    pub graph: ArcSwap<GraphSnapshot>,
    pub page_contents: DashMap<String, WikiPageContent>,
    pub section_corpus: ArcSwap<Vec<SectionDoc>>,
    pub bm25_index: ArcSwap<Bm25Index>,
    pub vector_registry: ArcSwap<HashMap<String, Vec<f32>>>,
    pub source_registry: RwLock<HashMap<String, SourceEntry>>,
    pub content_hashes: RwLock<HashMap<String, String>>,
    pub stale_flag: AtomicBool,
    pub config: RwLock<ProjectConfig>,
    pub audit_sender: tokio::sync::mpsc::Sender<AuditEvent>,
    pub audit_drops: AtomicU64,
    pub started_at: Instant,
    pub project_root: RwLock<PathBuf>,
    pub embedder: Box<dyn Embedder + Send + Sync>,
    pub vector_store: VectorStore,
    pub skill_engine: std::sync::RwLock<crate::skill::SkillEngine>,
    pub wiki_dir_mtime: std::sync::Mutex<Option<std::time::SystemTime>>,
    pub write_channel: WriteChannel,
    pub index_scheduler: IndexScheduler,
    pub session_memory: DashMap<String, MemoryEntry>,
    #[cfg(feature = "lsp")]
    pub lsp: Arc<wm_lsp::LspManager>,
    pub tool_list: RwLock<Vec<Tool>>,
}

impl EngineState {
    pub fn new(config: ProjectConfig, project_root: PathBuf) -> (Self, tokio::sync::mpsc::Receiver<AuditEvent>) {
        let (audit_sender, audit_receiver) = tokio::sync::mpsc::channel(1024);
        let (embedder, vector_store) = init_embedder(&config, &project_root);
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
                vector_registry: ArcSwap::new(Arc::new(HashMap::new())),
                source_registry: RwLock::new(HashMap::new()),
                content_hashes: RwLock::new(HashMap::new()),
                stale_flag: AtomicBool::new(true),
                config: RwLock::new(config),
                audit_sender,
                audit_drops: AtomicU64::new(0),
                started_at: Instant::now(),
                #[cfg(feature = "lsp")]
                lsp: {
                    let root_str = project_root.to_string_lossy().to_string();
                    Arc::new(wm_lsp::LspManager::new(&root_str))
                },
                project_root: RwLock::new(project_root),
                embedder,
                vector_store,
                wiki_dir_mtime: std::sync::Mutex::new(None),
                skill_engine: std::sync::RwLock::new(crate::skill::SkillEngine::new()),
                write_channel,
                index_scheduler: IndexScheduler::new(debounce_ms),
                session_memory: DashMap::new(),
                tool_list: RwLock::new(Vec::new()),
            },
            audit_receiver,
        )
    }

    pub fn set_tool_list(&self, tools: Vec<Tool>) {
        if let Ok(mut list) = self.tool_list.write() {
            *list = tools;
        }
    }

    pub fn resolve_path(&self, relative: &Path) -> PathBuf {
        if let Ok(root) = self.project_root.read() {
            root.join(relative)
        } else {
            relative.to_path_buf()
        }
    }

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

    pub fn rebuild_graph(&self, wiki_dir: &Path) -> usize {
        let custom_types = match self.config.read() {
            Ok(cfg) => cfg.custom_edge_types.clone(),
            Err(_) => {
                tracing::error!("config lock poisoned in rebuild_graph");
                Vec::new()
            }
        };
        let count = crate::graph::rebuild_graph_snapshot(&self.graph, wiki_dir, &custom_types);
        self.update_wiki_mtime(wiki_dir);
        count
    }

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

    pub fn scan_skills(&self, skills_dir: &Path) {
        if let Ok(mut engine) = self.skill_engine.write() {
            engine.scan(skills_dir);
            tracing::info!("Loaded {} skill(s)", engine.list().len());
        }
    }

    pub fn fire_skill_event(&self, event: &crate::skill::TriggerEvent) -> Vec<crate::skill::Skill> {
        if let Ok(engine) = self.skill_engine.read() {
            engine.fire_event(event).into_iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

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

    pub fn notify_file_changed(&self, _path: &Path) {
        #[cfg(feature = "lsp")]
        if let Some(content) = std::fs::read_to_string(_path).ok() {
            let lsp = self.lsp.clone();
            let path = _path.to_path_buf();
            tokio::spawn(async move {
                lsp.notify_file_changed(&path, &content).await;
            });
        }
    }
}
