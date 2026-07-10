use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::transport::ToolRegistry;
use crate::mcp::typed::TypedRegister;

/// Directories to skip when scanning source code.
const SKIP_DIRS: &[&str] = &[
    ".wm", ".agent", ".agents", ".knowns", ".git", ".github",
    ".claude", ".opencode", ".vscode", ".idea",
    "node_modules", "target",
];

/// Check if a directory name should be skipped during code search.
fn is_skipped_dir(name: &str) -> bool {
    SKIP_DIRS.contains(&name)
}

// ─── Input types ────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
struct WmCodeSearchInput {
    #[schemars(description = "Text pattern to search for")]
    pattern: String,
    #[schemars(description = "Subdirectory to search (relative to project root)")]
    path: Option<String>,
    #[schemars(description = "File extension filter (e.g. rs, ts, md)")]
    file_type: Option<String>,
    #[schemars(description = "Maximum results")]
    max_results: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
struct WmCodeSymbolsInput {
    #[schemars(description = "Filter by symbol name (substring)")]
    name: Option<String>,
    #[schemars(description = "Filter by symbol kind: function/struct/enum/trait/impl/module/type/const/macro")]
    kind: Option<String>,
    #[schemars(description = "Subdirectory filter")]
    path: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct WmCodeDepsInput {
    #[schemars(description = "Filter by specific file path")]
    file: Option<String>,
    #[schemars(description = "Dependency depth")]
    depth: Option<usize>,
}

/// Register code intelligence tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    // ─── wm_code.search ─────────────────────────────────────────
    let e = engine.clone();
    registry.register_read(
        "wm_code.search",
        "Search source code files by text pattern",
        move |input: WmCodeSearchInput| {
            let pattern = input.pattern;
            let sub_path = input.path;
            let file_type = input.file_type;
            let max_results = input.max_results.unwrap_or(30);

            let re = regex::Regex::new(&pattern)
                .map_err(|e| ToolError::internal(format!("Invalid regex pattern: {}", e)))?;

            let root = e
                .project_root
                .read()
                .map_err(|_| ToolError::lock_poisoned("project_root"))?
                .clone();

            let base_dir = match sub_path {
                Some(p) => root.join(&p),
                None => root.clone(),
            };

            if !base_dir.exists() {
                return Ok(json!({
                    "results": [],
                    "total": 0,
                    "truncated": false,
                    "error": format!("Directory does not exist: {}", base_dir.display())
                }));
            }

            let mut results = Vec::new();

            for entry in walkdir::WalkDir::new(&base_dir)
                .into_iter()
                .filter_entry(|e| {
                    e.file_name()
                        .to_str()
                        .map(|s| !is_skipped_dir(s))
                        .unwrap_or(false)
                })
                .filter_map(|e| e.ok())
            {
                if !entry.file_type().is_file() {
                    continue;
                }

                // Filter by file extension
                if let Some(ref ft) = file_type {
                    let ext = entry
                        .path()
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    if !ext.eq_ignore_ascii_case(ft) {
                        continue;
                    }
                }

                let content = match std::fs::read_to_string(entry.path()) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let file_path = entry.path().to_string_lossy().to_string();

                for (line_num, line) in content.lines().enumerate() {
                    if let Some(mat) = re.find(line) {
                        results.push(json!({
                            "file": file_path,
                            "line_number": line_num + 1,
                            "line": line,
                            "match": mat.as_str().to_string(),
                        }));
                        if results.len() >= max_results {
                            break;
                        }
                    }
                }

                if results.len() >= max_results {
                    break;
                }
            }

            let total = results.len();
            let truncated = total >= max_results;

            Ok(json!({
                "results": results,
                "total": total,
                "truncated": truncated,
            }))
        },
    );

    // ─── wm_code.symbols ─────────────────────────────────────────
    let e = engine.clone();
    registry.register_read(
        "wm_code.symbols",
        "Find symbol definitions (functions, structs, enums, traits, impls) in Rust source files",
        move |input: WmCodeSymbolsInput| {
            let filter_name = input.name;
            let filter_kind = input.kind;
            let sub_path = input.path;

            let root = e
                .project_root
                .read()
                .map_err(|_| ToolError::lock_poisoned("project_root"))?
                .clone();

            let base_dir = match sub_path {
                Some(p) => root.join(&p),
                None => root.clone(),
            };

            if !base_dir.exists() {
                return Ok(json!({
                    "symbols": [],
                    "total": 0,
                }));
            }

            // Symbol regex patterns: (regex, kind)
            let symbol_patterns: &[(&str, &str)] = &[
                (r"pub async fn (\w+)", "function"),
                (r"pub fn (\w+)", "function"),
                (r"pub struct (\w+)", "struct"),
                (r"pub enum (\w+)", "enum"),
                (r"pub unsafe trait (\w+)", "trait"),
                (r"pub trait (\w+)", "trait"),
                (r"impl.* for (\w+)", "impl"),
                (r"impl (\w+)", "impl"),
                (r"pub mod (\w+)", "module"),
                (r"mod (\w+)", "module"),
                (r"pub type (\w+)", "type"),
                (r"pub const (\w+)", "const"),
                (r"pub static (\w+)", "const"),
                (r"pub macro (\w+)", "macro"),
                (r"pub union (\w+)", "union"),
            ];

            let compiled: Vec<(regex::Regex, &str)> = symbol_patterns
                .iter()
                .map(|(pat, kind)| (regex::Regex::new(pat).unwrap(), *kind))
                .collect();

            let mut symbols = Vec::new();

            for entry in walkdir::WalkDir::new(&base_dir)
                .into_iter()
                .filter_entry(|e| {
                    e.file_name()
                        .to_str()
                        .map(|s| !is_skipped_dir(s))
                        .unwrap_or(false)
                })
                .filter_map(|e| e.ok())
            {
                if !entry.file_type().is_file() {
                    continue;
                }

                // Only process Rust files
                if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
                    continue;
                }

                let content = match std::fs::read_to_string(entry.path()) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let file_path = entry.path().to_string_lossy().to_string();

                for (line_num, line) in content.lines().enumerate() {
                    for (re, kind) in &compiled {
                        if let Some(caps) = re.captures(line) {
                            if let Some(name_match) = caps.get(1) {
                                let sym_name = name_match.as_str().to_string();

                                // Apply name filter (substring match)
                                if let Some(ref fname) = filter_name {
                                    if !sym_name.contains(fname.as_str()) {
                                        continue;
                                    }
                                }

                                // Apply kind filter
                                if let Some(ref fkind) = filter_kind {
                                    if *kind != fkind.as_str() {
                                        continue;
                                    }
                                }

                                symbols.push(json!({
                                    "name": sym_name,
                                    "kind": kind,
                                    "file": file_path.clone(),
                                    "line": line_num + 1,
                                    "snippet": line.trim().to_string(),
                                }));
                            }
                        }
                    }
                }
            }

            let total = symbols.len();
            Ok(json!({
                "symbols": symbols,
                "total": total,
            }))
        },
    );

    // ─── wm_code.deps ────────────────────────────────────────────
    let e = engine.clone();
    registry.register_read(
        "wm_code.deps",
        "Show import dependencies between files in the project",
        move |input: WmCodeDepsInput| {
            let filter_file = input.file;
            let _depth = input.depth.unwrap_or(1);

            let root = e
                .project_root
                .read()
                .map_err(|_| ToolError::lock_poisoned("project_root"))?
                .clone();

            let base_dir = root.clone();

            if !base_dir.exists() {
                return Ok(json!({
                    "dependencies": [],
                    "total": 0,
                }));
            }

            let use_re = regex::Regex::new(r"^\s*use\s+(.+);").unwrap();

            let mut dependencies = Vec::new();

            for entry in walkdir::WalkDir::new(&base_dir)
                .into_iter()
                .filter_entry(|e| {
                    e.file_name()
                        .to_str()
                        .map(|s| !is_skipped_dir(s))
                        .unwrap_or(false)
                })
                .filter_map(|e| e.ok())
            {
                if !entry.file_type().is_file() {
                    continue;
                }

                // Only process Rust files
                if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
                    continue;
                }

                let file_path = entry.path().to_string_lossy().to_string();

                // Apply file filter
                if let Some(ref ff) = filter_file {
                    if !file_path.contains(ff.as_str()) {
                        continue;
                    }
                }

                let content = match std::fs::read_to_string(entry.path()) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let mut deps = Vec::new();

                for (line_num, line) in content.lines().enumerate() {
                    if let Some(caps) = use_re.captures(line) {
                        if let Some(target) = caps.get(1) {
                            let use_path = target.as_str().trim().to_string();
                            if !use_path.is_empty() {
                                deps.push(json!({
                                    "target": use_path,
                                    "line": line_num + 1,
                                }));
                            }
                        }
                    }
                }

                if !deps.is_empty() {
                    dependencies.push(json!({
                        "file": file_path,
                        "deps": deps,
                    }));
                }
            }

            let total = dependencies.len();
            Ok(json!({
                "dependencies": dependencies,
                "total": total,
            }))
        },
    );
}
