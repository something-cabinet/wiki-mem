use crate::mcp::prelude::*;
use lsp_types::TextEdit;
use serde_json::json;
use wm_lsp::LspError;

#[derive(Deserialize, JsonSchema)]
pub struct DefinitionInput {
    #[schemars(description = "Absolute path to the file")]
    pub path: String,
    #[schemars(description = "Line number (0-indexed)")]
    pub line: u32,
    #[schemars(description = "Column number (0-indexed)")]
    pub col: u32,
}

#[derive(Deserialize, JsonSchema)]
pub struct ReferencesInput {
    #[schemars(description = "Absolute path to the file")]
    pub path: String,
    #[schemars(description = "Line number (0-indexed)")]
    pub line: u32,
    #[schemars(description = "Column number (0-indexed)")]
    pub col: u32,
    #[schemars(description = "Include declaration in references")]
    pub include_declaration: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
pub struct HoverInput {
    #[schemars(description = "Absolute path to the file")]
    pub path: String,
    #[schemars(description = "Line number (0-indexed)")]
    pub line: u32,
    #[schemars(description = "Column number (0-indexed)")]
    pub col: u32,
}

#[derive(Deserialize, JsonSchema)]
pub struct StatusInput {}

#[derive(Deserialize, JsonSchema)]
pub struct ImplementationsInput {
    #[schemars(description = "Absolute path to the file")]
    pub path: String,
    #[schemars(description = "Line number (0-indexed)")]
    pub line: u32,
    #[schemars(description = "Column number (0-indexed)")]
    pub col: u32,
}

#[derive(Deserialize, JsonSchema)]
pub struct WorkspaceSymbolsInput {
    #[schemars(description = "Search query for symbol names")]
    pub query: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct DiagnosticsInput {
    #[schemars(description = "Optional absolute path filter")]
    pub path: Option<String>,
    #[schemars(description = "Optional severity filter: error, warning, info, hint")]
    pub severity: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RenameInput {
    #[schemars(description = "Absolute path to the file")]
    pub path: String,
    #[schemars(description = "Line number (0-indexed)")]
    pub line: u32,
    #[schemars(description = "Column number (0-indexed)")]
    pub col: u32,
    #[schemars(description = "New name for the symbol")]
    pub new_name: String,
    #[schemars(
        description = "When true, apply the rename to disk; when false, return the edit plan"
    )]
    #[serde(default)]
    pub apply: bool,
}

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    #[cfg(feature = "lsp")]
    {
        let eng = engine.clone();
        registry.register_typed_async(
            "wm_lsp.definition",
            "Go to definition for a symbol at a given file position",
            move |input: DefinitionInput| {
                let eng = eng.clone();
                async move {
                    let lang = detect_language(&input.path)
                        .ok_or_else(|| ToolError::invalid_params("Unknown language for file"))?;
                    let server = eng.lsp.get_or_start(lang).await.map_err(to_tool_error)?;
                    let guard = server.write().await;
                    let uri = format!("file://{}", input.path);
                    let mut client = guard.client.lock().await;
                    if let Ok(text) = tokio::fs::read_to_string(&input.path).await {
                        client.did_open(&uri, &text, lang).await.ok();
                    }
                    let result = client
                        .definition(&uri, input.line, input.col)
                        .await
                        .map_err(to_tool_error)?;
                    Ok(json!({ "result": result }))
                }
            },
        );

        let eng = engine.clone();
        registry.register_typed_async(
            "wm_lsp.references",
            "Find references to a symbol at a given file position",
            move |input: ReferencesInput| {
                let eng = eng.clone();
                async move {
                    let lang = detect_language(&input.path)
                        .ok_or_else(|| ToolError::invalid_params("Unknown language for file"))?;
                    let server = eng.lsp.get_or_start(lang).await.map_err(to_tool_error)?;
                    let guard = server.write().await;
                    let uri = format!("file://{}", input.path);
                    let mut client = guard.client.lock().await;
                    if let Ok(text) = tokio::fs::read_to_string(&input.path).await {
                        client.did_open(&uri, &text, lang).await.ok();
                    }
                    let result = client
                        .references(
                            &uri,
                            input.line,
                            input.col,
                            input.include_declaration.unwrap_or(true),
                        )
                        .await
                        .map_err(to_tool_error)?;
                    Ok(json!({ "references": result }))
                }
            },
        );

        let eng = engine.clone();
        registry.register_typed_async(
            "wm_lsp.hover",
            "Hover over a symbol at a given file position to get type info and documentation",
            move |input: HoverInput| {
                let eng = eng.clone();
                async move {
                    let lang = detect_language(&input.path)
                        .ok_or_else(|| ToolError::invalid_params("Unknown language for file"))?;
                    let server = eng.lsp.get_or_start(lang).await.map_err(to_tool_error)?;
                    let guard = server.write().await;
                    let uri = format!("file://{}", input.path);
                    let mut client = guard.client.lock().await;
                    if let Ok(text) = tokio::fs::read_to_string(&input.path).await {
                        client.did_open(&uri, &text, lang).await.ok();
                    }
                    let result = client
                        .hover(&uri, input.line, input.col)
                        .await
                        .map_err(to_tool_error)?;
                    Ok(json!({ "hover": result }))
                }
            },
        );

        let engine_clone = engine.clone();
        registry.register_typed(
            "wm_lsp.status",
            "LSP server status per language",
            move |_: StatusInput| {
                let status = engine_clone.lsp.status();
                Ok(json!({ "servers": status }))
            },
        );

        let eng = engine.clone();
        registry.register_typed_async(
            "wm_lsp.implementations",
            "Find implementations of a symbol at a given file position",
            move |input: ImplementationsInput| {
                let eng = eng.clone();
                async move {
                    let lang = detect_language(&input.path)
                        .ok_or_else(|| ToolError::invalid_params("Unknown language for file"))?;
                    let server = eng.lsp.get_or_start(lang).await.map_err(to_tool_error)?;
                    let guard = server.write().await;
                    let uri = format!("file://{}", input.path);
                    let mut client = guard.client.lock().await;
                    if let Ok(text) = tokio::fs::read_to_string(&input.path).await {
                        client.did_open(&uri, &text, lang).await.ok();
                    }
                    let result = client
                        .goto_implementation(&uri, input.line, input.col)
                        .await
                        .map_err(to_tool_error)?;
                    Ok(json!({ "result": result }))
                }
            },
        );

        let eng = engine.clone();
        registry.register_typed_async(
            "wm_lsp.workspace_symbols",
            "Search workspace for symbols matching a query. Falls back to tree-sitter when LSP unavailable.",
            move |input: WorkspaceSymbolsInput| {
                let eng = eng.clone();
                async move {
                    let mut lsp_result: Option<Vec<serde_json::Value>> = None;
                    for lang in &["rust", "go", "typescript", "python"] {
                        if let Ok(server) = eng.lsp.get_or_start(lang).await {
                            let guard = server.write().await;
                            let mut client = guard.client.lock().await;
                            if let Ok(symbols) = client.workspace_symbol(&input.query).await
                            {
                                let items: Vec<_> = symbols
                                    .into_iter()
                                    .map(|s| {
                                        json!({
                                            "name": s.name,
                                            "kind": serde_json::json!(s.kind),
                                            "file": s.location.uri.as_str().to_string(),
                                            "line": s.location.range.start.line,
                                        })
                                    })
                                    .collect();
                                lsp_result = Some(items);
                                break;
                            }
                        }
                    }
                    if let Some(symbols) = lsp_result {
                        return Ok(json!({ "symbols": symbols }));
                    }

                    #[cfg_attr(not(feature = "code-intel"), allow(unused_mut))]
                    let mut fallback_symbols: Vec<serde_json::Value> = Vec::new();

                    #[cfg(feature = "code-intel")]
                    {
                        let root = match eng.project_root.read() {
                            Ok(r) => r.clone(),
                            Err(e) => return Err(ToolError::internal(e.to_string())),
                        };
                        let code_engine = crate::code_intel::CodeIntelEngine::global();
                        let exts = code_engine.supported_extensions();
                        let query_lower = input.query.to_lowercase();

                        for entry in walkdir::WalkDir::new(&root)
                            .into_iter()
                            .filter_map(|e| e.ok())
                            .filter(|e| e.file_type().is_file())
                        {
                            let path = entry.path();
                            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                                if exts.contains(&ext) {
                                    if let Ok(content) = tokio::fs::read_to_string(path).await {
                                        let syms = crate::code_intel::extract_symbols(
                                            &content,
                                            path.to_string_lossy().as_ref(),
                                            ext,
                                        );
                                        for s in syms {
                                            if query_lower.is_empty()
                                                || s.name
                                                    .to_lowercase()
                                                    .contains(&query_lower)
                                            {
                                                fallback_symbols.push(json!({
                                                    "name": s.name,
                                                    "kind": s.kind,
                                                    "file": s.file,
                                                    "line": s.line,
                                                }));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Ok(json!({ "symbols": fallback_symbols }))
                }
            },
        );

        let eng = engine.clone();
        registry.register_typed_async(
            "wm_lsp.diagnostics",
            "Get diagnostics from LSP servers. Optionally filter by path and/or severity.",
            move |input: DiagnosticsInput| {
                let eng = eng.clone();
                async move {
                    let mut results = Vec::new();
                    let severity_filter =
                        input
                            .severity
                            .as_deref()
                            .and_then(|s| match s.to_lowercase().as_str() {
                                "error" => Some(lsp_types::DiagnosticSeverity::ERROR),
                                "warning" => Some(lsp_types::DiagnosticSeverity::WARNING),
                                "info" => Some(lsp_types::DiagnosticSeverity::INFORMATION),
                                "hint" => Some(lsp_types::DiagnosticSeverity::HINT),
                                _ => None,
                            });

                    let languages: &[&str] = if let Some(ref path) = input.path {
                        if let Some(lang) = detect_language(path) {
                            &[lang][..]
                        } else {
                            &[]
                        }
                    } else {
                        &["rust", "go", "typescript", "python"]
                    };

                    for lang in languages {
                        if let Ok(server) = eng.lsp.get_or_start(lang).await {
                            let guard = server.read().await;
                            for entry in guard.diagnostics_cache.iter() {
                                let uri = entry.key();
                                let diagnostics = entry.value();

                                if let Some(ref path_filter) = input.path {
                                    let file_uri = format!("file://{}", path_filter);
                                    if *uri != file_uri {
                                        continue;
                                    }
                                }

                                for diag in diagnostics {
                                    if let Some(ref sev) = severity_filter {
                                        match diag.severity {
                                            Some(s) if s != *sev => continue,
                                            None => continue,
                                            _ => {}
                                        }
                                    }

                                    results.push(json!({
                                        "file": uri,
                                        "line": diag.range.start.line,
                                        "col": diag.range.start.character,
                                        "severity": diag.severity.map(|s| match s {
                                            lsp_types::DiagnosticSeverity::ERROR => "error",
                                            lsp_types::DiagnosticSeverity::WARNING => "warning",
                                            lsp_types::DiagnosticSeverity::INFORMATION => "info",
                                            lsp_types::DiagnosticSeverity::HINT => "hint",
                                            _ => "unknown",
                                        }),
                                        "message": diag.message,
                                        "source": diag.source,
                                    }));
                                }
                            }
                        }
                    }

                    Ok(json!({ "diagnostics": results }))
                }
            },
        );

        let eng = engine.clone();
        registry.register_typed_async(
            "wm_lsp.rename",
            "Rename a symbol at a given position. When apply=false (default), returns the edit plan. When apply=true, executes edits to disk.",
            move |input: RenameInput| {
                let eng = eng.clone();
                async move {
                    let lang = detect_language(&input.path)
                        .ok_or_else(|| ToolError::invalid_params("Unknown language for file"))?;
                    let server = eng
                        .lsp
                        .get_or_start(lang)
                        .await
                        .map_err(to_tool_error)?;
                    let guard = server.write().await;
                    let uri = format!("file://{}", input.path);
                    let mut client = guard.client.lock().await;
                    if let Ok(text) = tokio::fs::read_to_string(&input.path).await {
                        client.did_open(&uri, &text, lang).await.ok();
                    }
                    let edit = client.rename(&uri, input.line, input.col, &input.new_name).await.map_err(to_tool_error)?;

                    if !input.apply {
                        return Ok(json!({ "edit": edit }));
                    }

                    let count = apply_workspace_edit(&edit).await?;
                    Ok(json!({ "applied": count }))
                }
            },
        );
    }
}

async fn apply_workspace_edit(edit: &lsp_types::WorkspaceEdit) -> Result<usize, ToolError> {
    use lsp_types::{DocumentChanges, OneOf, TextEdit};
    use std::collections::HashSet;

    let mut files_changed: HashSet<String> = HashSet::new();

    if let Some(changes) = &edit.changes {
        for (uri, edits) in changes {
            let path = uri_to_path(uri.as_str())?;
            apply_text_edits(&path, edits).await?;
            files_changed.insert(path);
        }
    }

    if let Some(doc_changes) = &edit.document_changes {
        match doc_changes {
            DocumentChanges::Edits(edits_list) => {
                for doc_edit in edits_list {
                    let path = uri_to_path(doc_edit.text_document.uri.as_str())?;
                    let text_edits: Vec<TextEdit> = doc_edit
                        .edits
                        .iter()
                        .filter_map(|e| match e {
                            OneOf::Left(te) => Some(te.clone()),
                            _ => None,
                        })
                        .collect();
                    apply_text_edits(&path, &text_edits).await?;
                    files_changed.insert(path);
                }
            }
            DocumentChanges::Operations(_) => {
                return Err(ToolError::invalid_params(
                    "File operations in rename are not supported yet",
                ));
            }
        }
    }

    Ok(files_changed.len())
}

fn uri_to_path(uri: &str) -> Result<String, ToolError> {
    let path = uri
        .strip_prefix("file://")
        .ok_or_else(|| ToolError::invalid_params(format!("Expected file:// URI, got: {}", uri)))?;
    Ok(simple_percent_decode(path))
}

async fn apply_text_edits(path: &str, edits: &[TextEdit]) -> Result<(), ToolError> {
    if edits.is_empty() {
        return Ok(());
    }

    let mut content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| ToolError::io_error("read", path, e))?;

    let mut sorted_edits: Vec<&TextEdit> = edits.iter().collect();
    sorted_edits.sort_by(|a, b| {
        b.range
            .start
            .line
            .cmp(&a.range.start.line)
            .then_with(|| b.range.start.character.cmp(&a.range.start.character))
    });

    for edit in &sorted_edits {
        let start = byte_offset(&content, edit.range.start.line, edit.range.start.character);
        let end = byte_offset(&content, edit.range.end.line, edit.range.end.character);
        content.replace_range(start..end, &edit.new_text);
    }

    tokio::fs::write(path, &content)
        .await
        .map_err(|e| ToolError::io_error("write", path, e))?;
    Ok(())
}

fn byte_offset(content: &str, line: u32, character: u32) -> usize {
    content
        .split('\n')
        .take(line as usize)
        .map(|l| l.len() + 1)
        .sum::<usize>()
        .saturating_add(character as usize)
        .min(content.len())
}

fn simple_percent_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut bytes = s.bytes();
    while let Some(b) = bytes.next() {
        if b == b'%' {
            let hi = bytes.next().and_then(hex_val);
            let lo = bytes.next().and_then(hex_val);
            if let (Some(h), Some(l)) = (hi, lo) {
                out.push((h << 4 | l) as char);
            } else {
                out.push('%');
            }
        } else {
            out.push(b as char);
        }
    }
    out
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn detect_language(path: &str) -> Option<&'static str> {
    if path.ends_with(".rs") {
        Some("rust")
    } else if path.ends_with(".go") {
        Some("go")
    } else if path.ends_with(".ts") || path.ends_with(".tsx") || path.ends_with(".js") {
        Some("typescript")
    } else if path.ends_with(".py") {
        Some("python")
    } else {
        None
    }
}

fn to_tool_error(e: LspError) -> ToolError {
    match e {
        LspError::Unavailable {
            language,
            ref install_hint,
        } => ToolError::internal(format!("{} not available: {}", language, install_hint)),
        LspError::Starting => ToolError::internal("Language server starting"),
        LspError::Crashed { language } => {
            ToolError::internal(format!("{} language server crashed", language))
        }
        LspError::Timeout { operation } => {
            ToolError::internal(format!("LSP request timed out: {}", operation))
        }
        _ => ToolError::internal(e.to_string()),
    }
}
