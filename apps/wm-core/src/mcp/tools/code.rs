use crate::mcp::prelude::*;
use serde_json::json;


const SKIP_DIRS: &[&str] = &[
    ".wm", ".agent", ".agents", ".git", ".github",
    ".claude", ".opencode", ".vscode", ".idea",
    "node_modules", "target",
];

fn is_skipped_dir(name: &str) -> bool {
    SKIP_DIRS.contains(&name)
}

fn infer_lang_from_ext(ext: &str) -> &'static str {
    match ext {
        "rs" => "rust",
        "ts" => "typescript",
        "tsx" => "tsx",
        "js" => "javascript",
        "jsx" => "jsx",
        "mjs" => "javascript",
        "cjs" => "javascript",
        "py" => "python",
        "go" => "go",
        "html" | "htm" => "html",
        "svelte" => "svelte",
        "css" => "css",
        "scss" | "sass" => "scss",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "md" | "mdx" => "markdown",
        "sh" | "bash" | "zsh" => "bash",
        "sql" => "sql",
        "vue" => "vue",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "swift" => "swift",
        "c" | "h" => "c",
        "cpp" | "hpp" | "cc" | "cxx" => "cpp",
        "rb" => "ruby",
        "php" => "php",
        "scala" => "scala",
        "dart" => "dart",
        "lua" => "lua",
        "r" | "R" => "r",
        "zig" => "zig",
        "ex" | "exs" => "elixir",
        "erl" => "erlang",
        "clj" | "cljs" | "cljc" => "clojure",
        _ => "text",
    }
}


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
    #[schemars(description = "Filter by symbol kind: function/struct/enum/trait/class/interface/type/method/module")]
    kind: Option<String>,
    #[schemars(description = "Subdirectory filter")]
    path: Option<String>,
    #[schemars(description = "Filter by language: rust/typescript/tsx/python/go/html/svelte")]
    language: Option<String>,
    #[schemars(description = "Filter by specific file path")]
    file: Option<String>,
    #[schemars(description = "Maximum number of results")]
    max_results: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
struct WmCodeFileInput {
    #[schemars(description = "Path to the file, relative to the project root")]
    path: String,
}

#[derive(Deserialize, JsonSchema)]
struct WmCodeDepsInput {
    #[schemars(description = "Filter by specific file path")]
    file: Option<String>,
    #[schemars(description = "Dependency depth")]
    depth: Option<usize>,
    #[schemars(description = "Filter by language: rust/typescript/tsx/python/go/html/svelte")]
    language: Option<String>,
    #[schemars(description = "When true, return files that reference the given file instead of its dependencies")]
    reverse: Option<bool>,
}

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_typed(
        "wm_code.search",
        "Search source code files by text pattern (uses regex, tree-sitter enabled for metadata)",
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

    let e = engine.clone();
    registry.register_typed(
        "wm_code.symbols",
        "Find symbol definitions (functions, structs, enums, traits, impls, classes, interfaces) using tree-sitter AST when available",
        move |input: WmCodeSymbolsInput| {
            let filter_name = input.name;
            let filter_kind = input.kind;
            let sub_path = input.path;
            let _filter_lang = input.language;
            let filter_file = input.file;
            let max_results = input.max_results;

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

            let has_tree_sitter = cfg!(feature = "code-intel");

            let mut symbols: Vec<serde_json::Value> = Vec::new();

            if has_tree_sitter {
                #[cfg(feature = "code-intel")]
                {
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

                        let ext = entry.path()
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("");

                        if !crate::code_intel::CodeIntelEngine::global().is_supported(ext) {
                            continue;
                        }

                        if let Some(ref fl) = _filter_lang {
                            let lang_name = crate::code_intel::CodeIntelEngine::global()
                                .infer_language_from_ext(ext)
                                .unwrap_or("");
                            if lang_name != fl.as_str() {
                                continue;
                            }
                        }

                        let file_path = entry.path().to_string_lossy().to_string();
                        if let Some(ref ff) = filter_file {
                            if !file_path.contains(ff.as_str()) {
                                continue;
                            }
                        }

                        let content = match std::fs::read_to_string(entry.path()) {
                            Ok(c) => c,
                            Err(_) => continue,
                        };

                        let syms = crate::code_intel::extract_symbols(&content, &file_path, ext);

                        for sym in syms {
                            if let Some(ref fname) = filter_name {
                                if !sym.name.contains(fname.as_str()) {
                                    continue;
                                }
                            }

                            if let Some(ref fkind) = filter_kind {
                                if sym.kind != fkind.as_str() {
                                    continue;
                                }
                            }

                            symbols.push(json!({
                                "name": sym.name,
                                "kind": sym.kind,
                                "file": sym.file,
                                "line": sym.line,
                                "column": sym.column,
                                "snippet": sym.snippet,
                                "language": sym.language,
                            }));
                        }
                    }
                }
            } else {
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
                    .map(|(pat, kind)| (regex::Regex::new(pat).expect("hardcoded symbol pattern should be valid"), *kind))
                    .collect();

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

                    if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
                        continue;
                    }

                    let file_path = entry.path().to_string_lossy().to_string();
                    if let Some(ref ff) = filter_file {
                        if !file_path.contains(ff.as_str()) {
                            continue;
                        }
                    }

                    let content = match std::fs::read_to_string(entry.path()) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };

                    for (line_num, line) in content.lines().enumerate() {
                        for (re, kind) in &compiled {
                            if let Some(caps) = re.captures(line) {
                                if let Some(name_match) = caps.get(1) {
                                    let sym_name = name_match.as_str().to_string();

                                    if let Some(ref fname) = filter_name {
                                        if !sym_name.contains(fname.as_str()) {
                                            continue;
                                        }
                                    }

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
                                        "language": "rust",
                                    }));
                                }
                            }
                        }
                    }
                }
            }

            if let Some(mr) = max_results {
                symbols.truncate(mr);
            }

            let total = symbols.len();
            Ok(json!({
                "symbols": symbols,
                "total": total,
            }))
        },
    );

    let e = engine.clone();
    registry.register_typed(
        "wm_code.deps",
        "Show import dependencies between files using tree-sitter AST when available",
        move |input: WmCodeDepsInput| {
            let filter_file = input.file;
            let _depth = input.depth.unwrap_or(1);
            let _filter_lang = input.language;
            let reverse = input.reverse.unwrap_or(false);

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

            let has_tree_sitter = cfg!(feature = "code-intel");

            let mut dependencies: Vec<serde_json::Value> = Vec::new();

            if has_tree_sitter {
                #[cfg(feature = "code-intel")]
                {
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

                        let ext = entry.path()
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("");

                        if !crate::code_intel::CodeIntelEngine::global().is_supported(ext) {
                            continue;
                        }

                        if let Some(ref fl) = _filter_lang {
                            let lang_name = crate::code_intel::CodeIntelEngine::global()
                                .infer_language_from_ext(ext)
                                .unwrap_or("");
                            if lang_name != fl.as_str() {
                                continue;
                            }
                        }

                        let file_path = entry.path().to_string_lossy().to_string();

                        if reverse {
                            if let Some(ref target_path) = filter_file {
                                let content = match std::fs::read_to_string(entry.path()) {
                                    Ok(c) => c,
                                    Err(_) => continue,
                                };

                                let deps = crate::code_intel::extract_deps(&content, ext);
                                let matching_deps: Vec<_> = deps.iter()
                                    .filter(|d| d.target.contains(target_path.as_str()))
                                    .map(|d| json!({
                                        "target": d.target,
                                        "line": d.line,
                                        "kind": d.kind,
                                    }))
                                    .collect();

                                if !matching_deps.is_empty() {
                                    dependencies.push(json!({
                                        "file": file_path,
                                        "deps": matching_deps,
                                    }));
                                }
                            }
                        } else {
                            if let Some(ref ff) = filter_file {
                                if !file_path.contains(ff.as_str()) {
                                    continue;
                                }
                            }

                            let content = match std::fs::read_to_string(entry.path()) {
                                Ok(c) => c,
                                Err(_) => continue,
                            };

                            let deps = crate::code_intel::extract_deps(&content, ext);

                            if !deps.is_empty() {
                                dependencies.push(json!({
                                    "file": file_path,
                                    "deps": deps.iter().map(|d| json!({
                                        "target": d.target,
                                        "line": d.line,
                                        "kind": d.kind,
                                    })).collect::<Vec<_>>(),
                                }));
                            }
                        }
                    }
                }
            } else {
                let use_re = regex::Regex::new(r"^\s*use\s+(.+);").expect("hardcoded import pattern should be valid");

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

                    if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
                        continue;
                    }

                    let file_path = entry.path().to_string_lossy().to_string();

                    if reverse {
                        if let Some(ref target_path) = filter_file {
                            let content = match std::fs::read_to_string(entry.path()) {
                                Ok(c) => c,
                                Err(_) => continue,
                            };

                            let mut matching_deps = Vec::new();

                            for (line_num, line) in content.lines().enumerate() {
                                if let Some(caps) = use_re.captures(line) {
                                    if let Some(target) = caps.get(1) {
                                        let use_path = target.as_str().trim().to_string();
                                        if !use_path.is_empty() && use_path.contains(target_path.as_str()) {
                                            matching_deps.push(json!({
                                                "target": use_path,
                                                "line": line_num + 1,
                                            }));
                                        }
                                    }
                                }
                            }

                            if !matching_deps.is_empty() {
                                dependencies.push(json!({
                                    "file": file_path,
                                    "deps": matching_deps,
                                }));
                            }
                        }
                    } else {
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
                }
            }

            let total = dependencies.len();
            Ok(json!({
                "dependencies": dependencies,
                "total": total,
            }))
        },
    );

    let e = engine.clone();
    registry.register_typed(
        "wm_code.file",
        "Read a file's content, confined to the project root. Returns the content and inferred language.",
        move |input: WmCodeFileInput| {
            let root = e
                .project_root
                .read()
                .map_err(|_| ToolError::lock_poisoned("project_root"))?
                .clone();

            let canonical_root = root.canonicalize().unwrap_or_else(|_| root.clone());

            let requested = std::path::Path::new(&input.path);
            let resolved = if requested.is_absolute() {
                requested.to_path_buf()
            } else {
                root.join(requested)
            };

            let canonical = match resolved.canonicalize() {
                Ok(p) => p,
                Err(_) => {
                    return Err(ToolError::invalid_params(format!("File not found or inaccessible: {}", input.path)));
                }
            };

            if !canonical.starts_with(&canonical_root) {
                return Err(ToolError::invalid_params("Access denied: path is outside the project root"));
            }

            if canonical.components().any(|c| {
                c.as_os_str().to_str().map_or(false, |s| s.starts_with('.') && s != ".")
            }) {
                return Err(ToolError::invalid_params("Access denied: dotfiles and hidden directories are not readable"));
            }

            if !canonical.is_file() {
                return Err(ToolError::invalid_params(format!("Not a file: {}", input.path)));
            }

            let content = match std::fs::read_to_string(&canonical) {
                Ok(c) => c,
                Err(e) => {
                    return Err(ToolError::internal(format!("Failed to read file: {}", e)));
                }
            };

            let ext = canonical
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            let language = infer_lang_from_ext(ext);

            Ok(json!({
                "content": content,
                "language": language,
            }))
        },
    );
}
