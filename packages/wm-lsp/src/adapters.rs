use lsp_types::ClientCapabilities;

pub struct LanguageAdapter {
    pub language_id: String,
    pub extensions: Vec<&'static str>,
    pub binary_name: &'static str,
    pub args: &'static [&'static str],
    pub capabilities: ClientCapabilities,
}

pub fn adapter_for(language: &str) -> Option<LanguageAdapter> {
    match language {
        "rust" => Some(LanguageAdapter {
            language_id: "rust".to_string(),
            extensions: vec!["rs"],
            binary_name: "rust-analyzer",
            args: &[],
            capabilities: ClientCapabilities::default(),
        }),
        "go" => Some(LanguageAdapter {
            language_id: "go".to_string(),
            extensions: vec!["go"],
            binary_name: "gopls",
            args: &[],
            capabilities: ClientCapabilities::default(),
        }),
        "typescript" => Some(LanguageAdapter {
            language_id: "typescript".to_string(),
            extensions: vec!["ts", "tsx", "js", "jsx"],
            binary_name: "typescript-language-server",
            args: &["--stdio"],
            capabilities: ClientCapabilities::default(),
        }),
        "python" => Some(LanguageAdapter {
            language_id: "python".to_string(),
            extensions: vec!["py"],
            binary_name: "pyright-langserver",
            args: &["--stdio"],
            capabilities: ClientCapabilities::default(),
        }),
        _ => None,
    }
}
