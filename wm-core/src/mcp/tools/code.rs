use std::sync::Arc;

use serde_json::json;

use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;

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

/// Register code intelligence tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    // ─── wm_code.search ─────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_schema(
        "code.search",
        "Search source code files by text pattern",
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Text pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Subdirectory to search (relative to project root)"
                },
                "file_type": {
                    "type": "string",
                    "description": "File extension filter (e.g. rs, ts, md)"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum results",
                    "default": 30
                }
            },
            "required": ["pattern"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let pattern = args.require_string("pattern")?;
            let sub_path = args.optional_string("path");
            let file_type = args.optional_string("file_type");
            let max_results = args.optional_int("max_results").unwrap_or(30);

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
        }),
    );

    // ─── wm_code.symbols ─────────────────────────────────────────
    let e = engine.clone();
    registry.register_with_schema(
        "code.symbols",
        "Find symbol definitions (functions, structs, enums, traits, impls) in Rust source files",
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Filter by symbol name (substring)"
                },
                "kind": {
                    "type": "string",
                    "description": "Filter by symbol kind: function/struct/enum/trait/impl/module/type/const/macro"
                },
                "path": {
                    "type": "string",
                    "description": "Subdirectory filter"
                }
            }
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let filter_name = args.optional_string("name");
            let filter_kind = args.optional_string("kind");
            let sub_path = args.optional_string("path");

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
        }),
    );

    // ─── wm_code.deps ────────────────────────────────────────────
    registry.register_with_schema(
        "code.deps",
        "Show import dependencies between files in the project",
        json!({
            "type": "object",
            "properties": {
                "file": {
                    "type": "string",
                    "description": "Filter by specific file path"
                },
                "depth": {
                    "type": "integer",
                    "description": "Dependency depth",
                    "default": 1
                }
            }
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let filter_file = args.optional_string("file");
            let _depth = args.optional_int("depth").unwrap_or(1);

            let root = engine
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
        }),
    );
}
