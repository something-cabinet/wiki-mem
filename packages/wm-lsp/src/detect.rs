use crate::LspError;

pub struct LsBinary {
    pub command: String,
    pub args: Vec<String>,
    pub install_hint: String,
}

/// Find a language server binary for the given language
pub fn detect(language: &str) -> Result<LsBinary, LspError> {
    match language {
        "rust" => {
            // rust-analyzer is typically installed via rustup
            if let Ok(path) = which("rust-analyzer") {
                Ok(LsBinary {
                    command: path,
                    args: vec![],
                    install_hint: String::new(),
                })
            } else if let Ok(path) = find_in_rustup() {
                Ok(LsBinary {
                    command: path,
                    args: vec![],
                    install_hint: String::new(),
                })
            } else {
                Err(LspError::Unavailable {
                    language: "rust".into(),
                    install_hint: "Install rust-analyzer: rustup component add rust-analyzer"
                        .into(),
                })
            }
        }
        "go" => {
            if let Ok(path) = which("gopls") {
                Ok(LsBinary {
                    command: path,
                    args: vec![],
                    install_hint: String::new(),
                })
            } else {
                Err(LspError::Unavailable {
                    language: "go".into(),
                    install_hint: "Install gopls: go install golang.org/x/tools/gopls@latest"
                        .into(),
                })
            }
        }
        "typescript" => {
            if let Ok(path) = which("typescript-language-server") {
                Ok(LsBinary {
                    command: path,
                    args: vec!["--stdio".into()],
                    install_hint: String::new(),
                })
            } else {
                Err(LspError::Unavailable {
                    language: "typescript".into(),
                    install_hint: "Install: npm install -g typescript-language-server typescript"
                        .into(),
                })
            }
        }
        "python" => {
            if let Ok(path) = which("pyright-langserver") {
                Ok(LsBinary {
                    command: path,
                    args: vec!["--stdio".into()],
                    install_hint: String::new(),
                })
            } else if let Ok(path) = which("pylsp") {
                Ok(LsBinary {
                    command: path,
                    args: vec![],
                    install_hint: String::new(),
                })
            } else {
                Err(LspError::Unavailable {
                    language: "python".into(),
                    install_hint: "Install pyright: npm install -g pyright".into(),
                })
            }
        }
        _ => Err(LspError::Unavailable {
            language: language.to_string(),
            install_hint: format!("No LSP adapter for: {}", language),
        }),
    }
}

fn which(name: &str) -> Result<String, ()> {
    std::env::var_os("PATH")
        .and_then(|paths| {
            std::env::split_paths(&paths).find_map(|dir| {
                let full = dir.join(name);
                if full.exists() {
                    Some(full.to_string_lossy().to_string())
                } else {
                    let full_exe = dir.join(format!("{}.exe", name));
                    if full_exe.exists() {
                        Some(full_exe.to_string_lossy().to_string())
                    } else {
                        None
                    }
                }
            })
        })
        .ok_or(())
}

fn find_in_rustup() -> Result<String, ()> {
    // Check common rustup locations
    let home = std::env::var("HOME").unwrap_or_default();
    let candidates = vec![
        format!(
            "{}/.rustup/toolchains/stable-aarch64-apple-darwin/bin/rust-analyzer",
            home
        ),
        format!(
            "{}/.rustup/toolchains/stable-x86_64-apple-darwin/bin/rust-analyzer",
            home
        ),
        format!(
            "{}/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rust-analyzer",
            home
        ),
    ];
    candidates
        .into_iter()
        .find(|p| std::path::Path::new(p).exists())
        .ok_or(())
}
