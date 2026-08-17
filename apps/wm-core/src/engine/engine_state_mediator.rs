use super::index_scheduler_service::IndexScheduler;
use super::main_engine_factory::init_embedder;
use super::write_channel_proxy::WriteChannel;
use super::{
    AuditEvent, GraphEdge, GraphSnapshot, MemoryEntry, SectionDoc, SourceEntry, WikiPageContent,
    WikiPageMeta,
};
use crate::config::ProjectConfig;
use crate::search::Bm25Index;
use arc_swap::ArcSwap;
use dashmap::DashMap;
use petgraph::stable_graph::StableGraph;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use wm_constants::*;
use wm_embed::{Embedder, VectorStore};

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
    pub write_channel: WriteChannel,
    pub index_scheduler: IndexScheduler,
    pub session_memory: DashMap<String, MemoryEntry>,
    #[cfg(feature = "lsp")]
    pub lsp: Arc<wm_lsp::LspManager>,
    /// Serialized tool list (`name`/`description`/`inputSchema`), populated by
    /// `register_all_tools` for the `wm_help` tool. Transport-neutral JSON so
    /// wm-core does not need the optional rmcp dependency.
    pub tool_list: RwLock<Vec<Value>>,
}

impl EngineState {
    pub fn new(
        config: ProjectConfig,
        project_root: PathBuf,
    ) -> (Self, tokio::sync::mpsc::Receiver<AuditEvent>) {
        let (audit_sender, audit_receiver) = tokio::sync::mpsc::channel(AUDIT_CHANNEL_BUFFER);
        let (embedder, vector_store) = init_embedder(&config, &project_root);
        let (write_channel, write_receiver) = WriteChannel::new();
        let _write_handle = WriteChannel::spawn_consumer(write_receiver, project_root.clone());
        let debounce_ms = config.search.scoring.debounce_ms;
        (
            Self {
                graph: ArcSwap::new(Arc::new((
                    StableGraph::<WikiPageMeta, GraphEdge>::new(),
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
                skill_engine: std::sync::RwLock::new(crate::skill::SkillEngine::new()),
                write_channel,
                index_scheduler: IndexScheduler::new(debounce_ms),
                session_memory: DashMap::new(),
                tool_list: RwLock::new(Vec::new()),
            },
            audit_receiver,
        )
    }

    pub fn set_tool_list(&self, tools: Vec<Value>) {
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

    pub fn rebuild_graph(&self, wiki_dir: &Path) -> usize {
        let custom_types = match self.config.read() {
            Ok(cfg) => cfg.custom_edge_types.clone(),
            Err(_) => {
                tracing::error!("config lock poisoned in rebuild_graph");
                Vec::new()
            }
        };
        crate::graph::rebuild_graph_snapshot(&self.graph, wiki_dir, &custom_types)
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
        {
            let lsp = self.lsp.clone();
            let path = _path.to_path_buf();
            let notify = async move {
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    lsp.notify_file_changed(&path, &content).await;
                }
            };
            match tokio::runtime::Handle::try_current() {
                Ok(_) => {
                    tokio::spawn(notify);
                }
                Err(_) => {
                    std::thread::spawn(move || {
                        if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                        {
                            rt.block_on(notify);
                        }
                    });
                }
            }
        }
    }
}
