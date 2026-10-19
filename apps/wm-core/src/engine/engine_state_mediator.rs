use super::main_engine_factory::init_embedder;
use super::state::{
    AuditState, GraphState, IndexState, LifecycleState, MemoryState, PageState, SourceRegistryState,
};
use super::write_channel_proxy::WriteChannel;
use super::AuditEvent;
use crate::config::ProjectConfig;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};
use wm_constants::*;

pub struct EngineState {
    pub graph: GraphState,
    pub page: PageState,
    pub index: IndexState,
    pub source_registry: SourceRegistryState,
    pub stale_flag: AtomicBool,
    pub config: RwLock<ProjectConfig>,
    pub audit: AuditState,
    pub project_root: RwLock<PathBuf>,
    pub lifecycle: LifecycleState,
    pub session_memory: MemoryState,
    #[cfg(feature = "lsp")]
    pub lsp: Arc<wm_lsp::LspManager>,
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
                graph: GraphState::new(),
                page: PageState::new(),
                index: IndexState::new(embedder, vector_store),
                source_registry: SourceRegistryState::new(),
                stale_flag: AtomicBool::new(true),
                config: RwLock::new(config),
                audit: AuditState::new(audit_sender),
                #[cfg(feature = "lsp")]
                lsp: {
                    let root_str = project_root.to_string_lossy().to_string();
                    Arc::new(wm_lsp::LspManager::new(&root_str))
                },
                project_root: RwLock::new(project_root),
                lifecycle: LifecycleState::new(write_channel, debounce_ms),
                session_memory: MemoryState::new(),
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
        self.graph.rebuild(wiki_dir, &custom_types)
    }

    pub fn scan_skills(&self, skills_dir: &Path) {
        self.lifecycle.scan_skills(skills_dir);
    }

    pub fn fire_skill_event(&self, event: &crate::skill::TriggerEvent) -> Vec<crate::skill::Skill> {
        self.lifecycle.fire_skill_event(event).unwrap_or_default()
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
        self.audit.emit(event);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn emit_audit_routes_through_the_audit_handle() {
        let dir = tempfile::tempdir().expect("temp dir");
        let (engine, mut receiver) =
            EngineState::new(ProjectConfig::default(), dir.path().to_path_buf());

        engine.emit_audit(
            "page.recover",
            "auto-close",
            "ok",
            0,
            None,
            vec!["wiki:tasks:es-01".to_string()],
        );

        let event = receiver
            .try_recv()
            .expect("audit event reaches the receiver");
        assert_eq!(event.tool_name, "page.recover");
        assert_eq!(event.action, "auto-close");
        assert_eq!(event.result, "ok");
        assert_eq!(event.entity_refs, vec!["wiki:tasks:es-01".to_string()]);
        assert_eq!(engine.audit.dropped_count(), 0);
    }
}
