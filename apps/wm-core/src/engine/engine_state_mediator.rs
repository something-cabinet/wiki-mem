use super::main_engine_factory::init_embedder;
#[cfg(feature = "lsp")]
use super::state::LspState;
use super::state::{
    AuditState, GraphState, IndexState, LifecycleState, MemoryState, SourceRegistryState,
};
use super::write_channel_proxy::WriteChannel;
use super::AuditEvent;
use crate::config::ProjectConfig;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::RwLock;
use wm_constants::*;

pub struct EngineState {
    pub graph: GraphState,
    pub index: IndexState,
    pub source_registry: SourceRegistryState,
    pub stale_flag: AtomicBool,
    pub config: RwLock<ProjectConfig>,
    pub audit: AuditState,
    pub project_root: RwLock<PathBuf>,
    pub lifecycle: LifecycleState,
    pub session_memory: MemoryState,
    #[cfg(feature = "lsp")]
    pub lsp: LspState,
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
                index: IndexState::new(embedder, vector_store),
                source_registry: SourceRegistryState::new(),
                stale_flag: AtomicBool::new(true),
                config: RwLock::new(config),
                audit: AuditState::new(audit_sender),
                #[cfg(feature = "lsp")]
                lsp: LspState::new(&project_root.to_string_lossy()),
                project_root: RwLock::new(project_root),
                lifecycle: LifecycleState::new(write_channel, debounce_ms),
                session_memory: MemoryState::new(),
            },
            audit_receiver,
        )
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
    use std::collections::BTreeSet;

    const ENGINE_STATE_SOURCE: &str = include_str!("engine_state_mediator.rs");
    const LSP_FEATURE_GATE: &str = "#[cfg(feature = \"lsp\")]";
    const ENGINE_STATE_HANDLE_FIELDS: [&str; 6] = [
        "graph",
        "index",
        "source_registry",
        "audit",
        "lifecycle",
        "session_memory",
    ];
    #[cfg(feature = "lsp")]
    const ENGINE_STATE_LSP_HANDLE_FIELDS: [&str; 1] = ["lsp"];
    #[cfg(not(feature = "lsp"))]
    const ENGINE_STATE_LSP_HANDLE_FIELDS: [&str; 0] = [];
    const ENGINE_STATE_FLAT_FIELDS: [&str; 3] = ["stale_flag", "config", "project_root"];
    const ENGINE_STATE_MOVED_FIELDS: [&str; 2] = ["page", "tool_list"];

    fn engine_state_field_names() -> BTreeSet<String> {
        let start = ENGINE_STATE_SOURCE
            .find("pub struct EngineState {")
            .expect("EngineState declaration exists");
        let body = &ENGINE_STATE_SOURCE[start..];
        let end = body
            .find("\n}")
            .expect("EngineState declaration terminates");
        let mut fields = BTreeSet::new();
        let mut gated = false;
        for line in body[..end].lines().map(str::trim) {
            if line == LSP_FEATURE_GATE {
                gated = true;
                continue;
            }
            let declaration = match line.strip_prefix("pub ") {
                Some(declaration) => declaration,
                None => {
                    gated = false;
                    continue;
                }
            };
            if gated && !cfg!(feature = "lsp") {
                gated = false;
                continue;
            }
            let name = match declaration.split_once(':') {
                Some((name, _)) => name,
                None => {
                    gated = false;
                    continue;
                }
            };
            fields.insert(name.trim().to_string());
            gated = false;
        }
        fields
    }

    fn expected_engine_state_fields() -> BTreeSet<String> {
        ENGINE_STATE_HANDLE_FIELDS
            .iter()
            .chain(ENGINE_STATE_LSP_HANDLE_FIELDS.iter())
            .chain(ENGINE_STATE_FLAT_FIELDS.iter())
            .map(|name| (*name).to_string())
            .collect()
    }

    #[test]
    fn test_engine_state_fields_are_accounted_for_once() {
        let fields = engine_state_field_names();
        let handles: BTreeSet<String> = ENGINE_STATE_HANDLE_FIELDS
            .iter()
            .chain(ENGINE_STATE_LSP_HANDLE_FIELDS.iter())
            .map(|name| (*name).to_string())
            .collect();
        let flat: BTreeSet<String> = fields.difference(&handles).cloned().collect();
        let expected_flat: BTreeSet<String> = ENGINE_STATE_FLAT_FIELDS
            .iter()
            .map(|name| (*name).to_string())
            .collect();

        assert_eq!(
            flat, expected_flat,
            "flat EngineState fields drifted from the three remaining originals"
        );
        assert_eq!(
            fields,
            expected_engine_state_fields(),
            "EngineState must declare exactly the declared handles plus the three flat fields"
        );
        for field in ENGINE_STATE_MOVED_FIELDS {
            assert!(
                !fields.contains(field),
                "{field} moved to wm-tools and may not return to EngineState"
            );
        }
    }

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
