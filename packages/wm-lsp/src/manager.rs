use std::path::Path;
use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::RwLock;
use crate::{LspError, server::LspServer};

#[derive(Clone, Debug, serde::Serialize)]
pub struct ServerStatus {
    pub language: String,
    pub enabled: bool,
    pub binary_found: bool,
    pub running: bool,
    pub ready: bool,
    pub install_hint: Option<String>,
}

pub struct LspManager {
    servers: DashMap<String, Arc<RwLock<LspServer>>>,
    project_root: String,
}

impl LspManager {
    pub fn new(project_root: &str) -> Self {
        Self { servers: DashMap::new(), project_root: project_root.to_string() }
    }

    pub async fn get_or_start(&self, language: &str) -> Result<Arc<RwLock<LspServer>>, LspError> {
        if let Some(server) = self.servers.get(language) {
            return Ok(server.value().clone());
        }

        let server = start_server(language, &self.project_root).await?;
        let server = Arc::new(RwLock::new(server));
        self.servers.insert(language.to_string(), server.clone());
        Ok(server)
    }

    pub fn status(&self) -> Vec<ServerStatus> {
        let languages = ["rust", "go", "typescript", "python"];
        languages.iter().map(|lang| {
            let running = self.servers.contains_key(*lang);
            ServerStatus {
                language: lang.to_string(),
                enabled: true,
                binary_found: true, // simplified: binary check TBD
                running,
                ready: running, // simplified
                install_hint: None,
            }
        }).collect()
    }

    pub async fn notify_file_changed(&self, _path: &Path, _content: &str) {
        // For now, no-op. Language-specific URI routing TBD.
    }
}

async fn start_server(language: &str, root: &str) -> Result<LspServer, LspError> {
    let root_uri = format!("file://{}", root);
    match language {
        "rust" => LspServer::start("rust-analyzer", &[], &root_uri, "rust").await,
        "go" => LspServer::start("gopls", &[], &root_uri, "go").await,
        "typescript" => LspServer::start("typescript-language-server", &["--stdio".to_string()], &root_uri, "typescript").await,
        "python" => LspServer::start("pyright-langserver", &["--stdio".to_string()], &root_uri, "python").await,
        _ => Err(LspError::Unavailable {
            language: language.to_string(),
            install_hint: format!("Unsupported language: {}", language),
        }),
    }
}
