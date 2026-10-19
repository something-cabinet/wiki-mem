use std::path::Path;
use std::sync::Arc;

use tokio::sync::RwLock;
use wm_lsp::{LspError, LspManager, LspServer, ServerStatus};

#[derive(Clone)]
pub struct LspState {
    manager: Arc<LspManager>,
}

impl LspState {
    pub fn new(project_root: &str) -> Self {
        Self {
            manager: Arc::new(LspManager::new(project_root)),
        }
    }

    pub async fn get_or_start(&self, language: &str) -> Result<Arc<RwLock<LspServer>>, LspError> {
        self.manager.get_or_start(language).await
    }

    pub fn status(&self) -> Vec<ServerStatus> {
        self.manager.status()
    }

    pub async fn notify_file_changed(&self, path: &Path, content: &str) {
        self.manager.notify_file_changed(path, content).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LSP_FIELDS: [&str; 1] = ["manager"];
    const EXPECTED_LANGUAGES: [&str; 4] = ["rust", "go", "typescript", "python"];
    const PROJECT_ROOT: &str = "/tmp/lsp-state-model";

    fn lsp_struct_body() -> &'static str {
        const SOURCE: &str = include_str!("lsp_state_model.rs");
        let start = SOURCE
            .find("pub struct LspState {")
            .expect("LspState declaration exists");
        let body = &SOURCE[start..];
        let end = body.find("\n}").expect("LspState body terminates");
        &body[..end]
    }

    fn declared_field_line(field: &str) -> &'static str {
        let needles = [format!("{field}:"), format!("pub {field}:")];
        let matches: Vec<&str> = lsp_struct_body()
            .lines()
            .map(str::trim)
            .filter(|line| {
                needles
                    .iter()
                    .any(|needle| line.starts_with(needle.as_str()))
            })
            .collect();
        assert_eq!(
            matches.len(),
            1,
            "expected exactly one declaration of {field}"
        );
        matches[0]
    }

    #[test]
    fn test_lsp_state_declares_its_manager_private() {
        for field in LSP_FIELDS {
            let declaration = declared_field_line(field);
            assert!(
                !declaration.starts_with("pub"),
                "{field} must stay private so no sibling handle can read it"
            );
        }
        assert_eq!(
            lsp_struct_body()
                .lines()
                .map(str::trim)
                .filter(|line| line.ends_with(','))
                .count(),
            LSP_FIELDS.len(),
            "LspState may not carry a sibling handle field"
        );
    }

    #[test]
    fn test_status_reports_the_supported_languages_before_any_start() {
        let lsp = LspState::new(PROJECT_ROOT);

        let status = lsp.status();
        let languages: Vec<&str> = status
            .iter()
            .map(|server| server.language.as_str())
            .collect();
        assert_eq!(languages, EXPECTED_LANGUAGES);
        assert!(status.iter().all(|server| !server.running));
    }
}
