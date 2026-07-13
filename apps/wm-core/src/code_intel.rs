use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Serialize;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor, Language as TsLanguage};

// ─── Types ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct CodeIntelSymbol {
    pub name: String,
    pub kind: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub snippet: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodeIntelDep {
    pub target: String,
    pub line: usize,
    pub kind: String,
}

#[derive(Debug, Clone)]
enum SupportedLanguage {
    Rust,
    TypeScript,
    Tsx,
    Python,
    Go,
    Html,
    Svelte,
}

impl SupportedLanguage {
    fn from_ext(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Self::Rust),
            "ts" => Some(Self::TypeScript),
            "tsx" => Some(Self::Tsx),
            "py" => Some(Self::Python),
            "go" => Some(Self::Go),
            "html" | "htm" => Some(Self::Html),
            "svelte" => Some(Self::Svelte),
            _ => None,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
            Self::Tsx => "tsx",
            Self::Python => "python",
            Self::Go => "go",
            Self::Html => "html",
            Self::Svelte => "svelte",
        }
    }

    fn load_language(&self) -> TsLanguage {
        match self {
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
            Self::Python => tree_sitter_python::LANGUAGE.into(),
            Self::Go => tree_sitter_go::LANGUAGE.into(),
            Self::Html => tree_sitter_html::LANGUAGE.into(),
            Self::Svelte => tree_sitter_svelte_ng::LANGUAGE.into(),
        }
    }
}

// ─── Engine ─────────────────────────────────────────────────────

static ENGINE: OnceLock<CodeIntelEngine> = OnceLock::new();

pub struct CodeIntelEngine {
    languages: HashMap<&'static str, SupportedLanguage>,
}

impl CodeIntelEngine {
    pub fn global() -> &'static CodeIntelEngine {
        ENGINE.get_or_init(|| CodeIntelEngine::new())
    }

    fn new() -> Self {
        let mut languages = HashMap::new();
        languages.insert("rs", SupportedLanguage::Rust);
        languages.insert("ts", SupportedLanguage::TypeScript);
        languages.insert("tsx", SupportedLanguage::Tsx);
        languages.insert("py", SupportedLanguage::Python);
        languages.insert("go", SupportedLanguage::Go);
        languages.insert("html", SupportedLanguage::Html);
        languages.insert("htm", SupportedLanguage::Html);
        languages.insert("svelte", SupportedLanguage::Svelte);
        Self { languages }
    }

    pub fn supported_extensions(&self) -> Vec<&'static str> {
        let mut exts: Vec<_> = self.languages.keys().copied().collect();
        exts.sort();
        exts
    }

    pub fn is_supported(&self, ext: &str) -> bool {
        self.languages.contains_key(ext)
    }

    pub fn infer_language_from_ext(&self, ext: &str) -> Option<&'static str> {
        SupportedLanguage::from_ext(ext).map(|l| l.name())
    }
}

// ─── Grammar introspection (debug/development only) ──────────

#[allow(dead_code)]
fn dump_node_structure(source: &str, ext: &str, max_depth: usize) -> String {
    let parser_mutex = match get_or_create_parser(ext) {
        Some(m) => m,
        None => return "unsupported".to_string(),
    };
    let mut parser = match parser_mutex.lock() {
        Ok(p) => p,
        Err(_) => return "lock error".to_string(),
    };
    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return "parse error".to_string(),
    };
    dump_node(tree.root_node(), source, 0, max_depth)
}

#[allow(dead_code)]
fn dump_node(node: tree_sitter::Node, source: &str, depth: usize, max_depth: usize) -> String {
    if depth > max_depth {
        return String::new();
    }
    let mut result = String::new();
    let kind = node.kind();
    let start = node.start_position().row + 1;
    let end = node.end_position().row + 1;
    let text = node.utf8_text(source.as_bytes()).unwrap_or("");
    let indent = "  ".repeat(depth);

    // Skip long text
    let snippet = if text.len() > 40 {
        format!("{}...", &text[..37])
    } else {
        text.to_string()
    };

    result.push_str(&format!("{}{} [{}:{}] \"{}\"\n", indent, kind, start, end, snippet));

    let mut child = node.walk();
    for c in node.children(&mut child) {
        result.push_str(&dump_node(c, source, depth + 1, max_depth));
    }

    result
}

// ─── Per-language parser storage ────────────────────────────────

fn get_or_create_parser(ext: &str) -> Option<&'static std::sync::Mutex<Parser>> {
    let lang = SupportedLanguage::from_ext(ext)?;
    let name = lang.name();

    // Use a static array of mutexes indexed by language name
    static RUST: OnceLock<std::sync::Mutex<Parser>> = OnceLock::new();
    static TS: OnceLock<std::sync::Mutex<Parser>> = OnceLock::new();
    static TSX: OnceLock<std::sync::Mutex<Parser>> = OnceLock::new();
    static PY: OnceLock<std::sync::Mutex<Parser>> = OnceLock::new();
    static GO: OnceLock<std::sync::Mutex<Parser>> = OnceLock::new();
    static HTML: OnceLock<std::sync::Mutex<Parser>> = OnceLock::new();
    static SVELTE: OnceLock<std::sync::Mutex<Parser>> = OnceLock::new();

    let cell: &OnceLock<std::sync::Mutex<Parser>> = match name {
        "rust" => &RUST,
        "typescript" => &TS,
        "tsx" => &TSX,
        "python" => &PY,
        "go" => &GO,
        "html" => &HTML,
        "svelte" => &SVELTE,
        _ => return None,
    };

    Some(cell.get_or_init(|| {
        let mut parser = Parser::new();
        let ts_lang = lang.load_language();
        parser.set_language(&ts_lang).expect("tree-sitter language setup");
        std::sync::Mutex::new(parser)
    }))
}

fn parse_source(source: &str, ext: &str) -> Result<tree_sitter::Tree, String> {
    let parser_mutex = get_or_create_parser(ext)
        .ok_or_else(|| format!("Unsupported extension: {}", ext))?;
    let mut parser = parser_mutex.lock().map_err(|e| format!("Parser lock error: {}", e))?;
    parser.parse(source, None).ok_or_else(|| "Failed to parse source".to_string())
}

// ─── Query helpers ──────────────────────────────────────────────

struct CompiledQuery {
    query: Query,
    name_index: u32,
}

fn compile_query(lang: &SupportedLanguage, query_str: &str, kind: &'static str) -> Result<CompiledQuery, String> {
    let ts_lang = lang.load_language();
    let query = Query::new(&ts_lang, query_str)
        .map_err(|e| format!("Failed to compile {} query: {}", kind, e))?;

    let name_index = query.capture_index_for_name("name")
        .ok_or_else(|| format!("Query missing @name capture for {}", kind))?;

    Ok(CompiledQuery { query, name_index })
}

fn run_query<'a>(
    query: &Query,
    name_index: u32,
    root: tree_sitter::Node<'a>,
    source: &'a [u8],
) -> Vec<(String, usize, usize, usize, usize)> {
    let mut cursor = QueryCursor::new();
    let mut query_matches = cursor.matches(query, root, source);
    let mut results = Vec::new();

    while let Some(match_) = query_matches.next() {
        for capture in match_.captures {
            if capture.index == name_index {
                let range = capture.node.range();
                let name = capture.node.utf8_text(source).unwrap_or("").to_string();
                let start_byte = range.start_byte;
                let start_line = range.start_point.row + 1;
                let start_col = range.start_point.column;

                if !name.is_empty() {
                    results.push((name, start_line, start_col, start_byte, range.end_byte));
                }
            }
        }
    }

    results
}

fn get_line_at_offset(source: &str, offset: usize) -> &str {
    let byte_idx = offset.min(source.len());
    let before = &source[..byte_idx];
    let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let after = &source[line_start..];
    after.lines().next().unwrap_or("")
}

// ─── Symbol queries per language ────────────────────────────────

fn symbols_for_rust(source: &str, file: &str, language: &str) -> Vec<CodeIntelSymbol> {
    let mut results = Vec::new();

    let queries: &[(&str, &str)] = &[
        (r"(function_item name: (identifier) @name)", "function"),
        (r"(function_signature name: (identifier) @name)", "function"),
        (r"(struct_item name: (type_identifier) @name)", "struct"),
        (r"(enum_item name: (type_identifier) @name)", "enum"),
        (r"(union_item name: (type_identifier) @name)", "union"),
        (r"(trait_item name: (type_identifier) @name)", "trait"),
        (r"(type_item name: (type_identifier) @name)", "type"),
        (r"(const_item name: (identifier) @name)", "const"),
        (r"(static_item name: (identifier) @name)", "const"),
        (r"(macro_definition name: (identifier) @name)", "macro"),
        (r"(mod_item name: (identifier) @name)", "module"),
        (r"(impl_item trait: [(type_identifier) (qualified_type)]? @name)", "impl"),
    ];

    let lang = &SupportedLanguage::Rust;

    let tree = match parse_source(source, "rs") {
        Ok(t) => t,
        Err(_) => return results,
    };

    for (query_str, kind) in queries {
        if let Ok(cq) = compile_query(lang, query_str, kind) {
            for (name, line, col, _sb, _eb) in run_query(&cq.query, cq.name_index, tree.root_node(), source.as_bytes()) {
                let snippet = get_line_at_offset(source, _sb).trim().to_string();
                results.push(CodeIntelSymbol {
                    name, kind: kind.to_string(), file: file.to_string(),
                    line, column: col, snippet, language: language.to_string(),
                });
            }
        }
    }

    results
}

fn symbols_for_typescript(source: &str, file: &str, language: &str, is_tsx: bool) -> Vec<CodeIntelSymbol> {
    let lang = if is_tsx { SupportedLanguage::Tsx } else { SupportedLanguage::TypeScript };
    let mut results = Vec::new();

    let queries: &[(&str, &str)] = &[
        (r"(function_declaration name: (identifier) @name)", "function"),
        (r"(method_definition name: (property_identifier) @name)", "method"),
        (r"(class_declaration name: (type_identifier) @name)", "class"),
        (r"(interface_declaration name: (type_identifier) @name)", "interface"),
        (r"(type_alias_declaration name: (type_identifier) @name)", "type"),
        (r"(enum_declaration name: (identifier) @name)", "enum"),
        (r"(abstract_method_signature name: (property_identifier) @name)", "method"),
        (r"(variable_declarator name: (identifier) @name)", "variable"),
    ];

    let ext = if is_tsx { "tsx" } else { "ts" };

    let tree = match parse_source(source, ext) {
        Ok(t) => t,
        Err(_) => return results,
    };

    for (query_str, kind) in queries {
        if let Ok(cq) = compile_query(&lang, query_str, kind) {
            for (name, line, col, _sb, _eb) in run_query(&cq.query, cq.name_index, tree.root_node(), source.as_bytes()) {
                let snippet = get_line_at_offset(source, _sb).trim().to_string();
                results.push(CodeIntelSymbol {
                    name, kind: kind.to_string(), file: file.to_string(),
                    line, column: col, snippet, language: language.to_string(),
                });
            }
        }
    }

    results
}

fn symbols_for_python(source: &str, file: &str, language: &str) -> Vec<CodeIntelSymbol> {
    let mut results = Vec::new();

    let queries: &[(&str, &str)] = &[
        (r"(function_definition name: (identifier) @name)", "function"),
        (r"(class_definition name: (identifier) @name)", "class"),
        (r"(decorated_definition (function_definition name: (identifier) @name))", "function"),
        (r"(decorated_definition (class_definition name: (identifier) @name))", "class"),
    ];

    let lang = &SupportedLanguage::Python;

    let tree = match parse_source(source, "py") {
        Ok(t) => t,
        Err(_) => return results,
    };

    for (query_str, kind) in queries {
        if let Ok(cq) = compile_query(lang, query_str, kind) {
            for (name, line, col, _sb, _eb) in run_query(&cq.query, cq.name_index, tree.root_node(), source.as_bytes()) {
                let snippet = get_line_at_offset(source, _sb).trim().to_string();
                results.push(CodeIntelSymbol {
                    name, kind: kind.to_string(), file: file.to_string(),
                    line, column: col, snippet, language: language.to_string(),
                });
            }
        }
    }

    results
}

fn symbols_for_go(source: &str, file: &str, language: &str) -> Vec<CodeIntelSymbol> {
    let mut results = Vec::new();

    let queries: &[(&str, &str)] = &[
        (r"(function_declaration name: (identifier) @name)", "function"),
        (r"(method_declaration name: (field_identifier) @name)", "method"),
        (r"(type_declaration (type_spec name: (type_identifier) @name))", "type"),
    ];

    let lang = &SupportedLanguage::Go;

    let tree = match parse_source(source, "go") {
        Ok(t) => t,
        Err(_) => return results,
    };

    for (query_str, kind) in queries {
        if let Ok(cq) = compile_query(lang, query_str, kind) {
            for (name, line, col, _sb, _eb) in run_query(&cq.query, cq.name_index, tree.root_node(), source.as_bytes()) {
                let snippet = get_line_at_offset(source, _sb).trim().to_string();
                results.push(CodeIntelSymbol {
                    name, kind: kind.to_string(), file: file.to_string(),
                    line, column: col, snippet, language: language.to_string(),
                });
            }
        }
    }

    results
}

fn symbols_for_html(source: &str, file: &str, language: &str) -> Vec<CodeIntelSymbol> {
    let mut results = Vec::new();

    let queries: &[(&str, &str)] = &[
        (r"(element (start_tag (tag_name) @name))", "element"),
        (r"(self_closing_tag (tag_name) @name)", "element"),
    ];

    let lang = &SupportedLanguage::Html;

    let tree = match parse_source(source, "html") {
        Ok(t) => t,
        Err(_) => return results,
    };

    for (query_str, kind) in queries {
        if let Ok(cq) = compile_query(lang, query_str, kind) {
            for (name, line, col, _sb, _eb) in run_query(&cq.query, cq.name_index, tree.root_node(), source.as_bytes()) {
                let snippet = get_line_at_offset(source, _sb).trim().to_string();
                results.push(CodeIntelSymbol {
                    name, kind: kind.to_string(), file: file.to_string(),
                    line, column: col, snippet, language: language.to_string(),
                });
            }
        }
    }

    results
}

fn symbols_for_svelte(source: &str, file: &str, language: &str) -> Vec<CodeIntelSymbol> {
    let mut results = Vec::new();

    let queries: &[(&str, &str)] = &[
        (r"(element (start_tag (tag_name) @name))", "element"),
        (r"(component (identifier) @name)", "component"),
    ];

    let lang = &SupportedLanguage::Svelte;

    let tree = match parse_source(source, "svelte") {
        Ok(t) => t,
        Err(_) => return results,
    };

    for (query_str, kind) in queries {
        if let Ok(cq) = compile_query(lang, query_str, kind) {
            for (name, line, col, _sb, _eb) in run_query(&cq.query, cq.name_index, tree.root_node(), source.as_bytes()) {
                let snippet = get_line_at_offset(source, _sb).trim().to_string();
                results.push(CodeIntelSymbol {
                    name, kind: kind.to_string(), file: file.to_string(),
                    line, column: col, snippet, language: language.to_string(),
                });
            }
        }
    }

    results
}

// ─── Public API ─────────────────────────────────────────────────

pub fn extract_symbols(source: &str, file: &str, ext: &str) -> Vec<CodeIntelSymbol> {
    let lang = match SupportedLanguage::from_ext(ext) {
        Some(l) => l,
        None => return Vec::new(),
    };

    let language_name = lang.name().to_string();

    match lang {
        SupportedLanguage::Rust => symbols_for_rust(source, file, &language_name),
        SupportedLanguage::TypeScript | SupportedLanguage::Tsx => {
            symbols_for_typescript(source, file, &language_name, matches!(lang, SupportedLanguage::Tsx))
        }
        SupportedLanguage::Python => symbols_for_python(source, file, &language_name),
        SupportedLanguage::Go => symbols_for_go(source, file, &language_name),
        SupportedLanguage::Html => symbols_for_html(source, file, &language_name),
        SupportedLanguage::Svelte => symbols_for_svelte(source, file, &language_name),
    }
}

pub fn extract_deps(source: &str, ext: &str) -> Vec<CodeIntelDep> {
    let lang = match SupportedLanguage::from_ext(ext) {
        Some(l) => l,
        None => return Vec::new(),
    };

    let query_str = match lang {
        SupportedLanguage::Rust => r"(use_declaration) @target",
        SupportedLanguage::TypeScript | SupportedLanguage::Tsx => {
            r"[
                (import_statement (string (string_fragment) @target))
            ]"
        }
        SupportedLanguage::Python => r"[
            (import_statement name: (dotted_name) @target)
            (import_from_statement module_name: (dotted_name) @target)
        ]",
        SupportedLanguage::Go => r"(import_spec path: (interpreted_string_literal) @target)",
        SupportedLanguage::Html | SupportedLanguage::Svelte => {
            return Vec::new();
        }
    };

    let ts_lang = lang.load_language();
    let query = match Query::new(&ts_lang, query_str) {
        Ok(q) => q,
        Err(_) => return Vec::new(),
    };

    let target_index = match query.capture_index_for_name("target") {
        Some(i) => i,
        None => return Vec::new(),
    };

    let tree = match parse_source(source, ext) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };

    let mut cursor = QueryCursor::new();
    let mut query_matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
    let mut results = Vec::new();

    while let Some(match_) = query_matches.next() {
        for capture in match_.captures {
            if capture.index == target_index {
                let range = capture.node.range();
                let mut target = capture.node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                // Clean up Rust use declarations: strip "use " prefix and ";" suffix
                if let SupportedLanguage::Rust = lang {
                    if let Some(rest) = target.strip_prefix("use ") {
                        target = rest.to_string();
                    }
                    if target.ends_with(';') {
                        target = target[..target.len() - 1].to_string();
                    }
                }
                if !target.is_empty() {
                    let dep_kind = match lang {
                        SupportedLanguage::Rust => "use",
                        SupportedLanguage::TypeScript | SupportedLanguage::Tsx => "import",
                        SupportedLanguage::Python => "import",
                        SupportedLanguage::Go => "import",
                        SupportedLanguage::Html | SupportedLanguage::Svelte => "",
                    };
                    results.push(CodeIntelDep {
                        target,
                        line: range.start_point.row + 1,
                        kind: dep_kind.to_string(),
                    });
                }
            }
        }
    }

    results
}

pub fn infer_language_from_ext(ext: &str) -> Option<&'static str> {
    SupportedLanguage::from_ext(ext).map(|l| l.name())
}

// ─── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_parser_basic() {
        let source = "fn hello() {}";
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        let root_kind = root.kind();
        let child_count = root.child_count();
        if root_kind != "source_file" || child_count == 0 {
            panic!("Rust parser failed: kind={}, children={}", root_kind, child_count);
        }
        // Check that we can match function_item
        let query = tree_sitter::Query::new(
            &tree_sitter_rust::LANGUAGE.into(),
            "(function_item name: (identifier) @name)",
        ).unwrap();
        let mut cursor = tree_sitter::QueryCursor::new();
        let mut matches = cursor.matches(&query, root, source.as_bytes());
        let mut count = 0;
        while let Some(_) = matches.next() {
            count += 1;
        }
        assert_eq!(count, 1, "Should find 1 function, found {}", count);
    }
    
    // ─── Rust ──────────────────────────────────────────────────

    #[test]
    fn test_rust_functions_and_structs() {
        let source = r#"
pub fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

pub struct User {
    name: String,
    age: u32,
}

pub enum Status {
    Active,
    Inactive,
}

pub trait Runnable {
    fn run(&self);
}

impl Runnable for User {
    fn run(&self) {
        println!("running");
    }
}

pub const MAX_RETRIES: u32 = 3;

mod utils;

pub type Callback = Box<dyn Fn()>;

macro_rules! define_impl {
    () => {};
}
"#;
        let syms = extract_symbols(source, "test.rs", "rs");
        assert!(!syms.is_empty(), "Should find symbols in Rust source");

        let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"hello"), "Should find function hello");
        assert!(names.contains(&"User"), "Should find struct User");
        assert!(names.contains(&"Status"), "Should find enum Status");
        assert!(names.contains(&"Runnable"), "Should find trait Runnable");
        assert!(names.contains(&"MAX_RETRIES"), "Should find const MAX_RETRIES");
        assert!(names.contains(&"Callback"), "Should find type Callback");
        assert!(names.contains(&"utils"), "Should find module utils");

        let kinds: Vec<&str> = syms.iter().map(|s| s.kind.as_str()).collect();
        assert!(kinds.contains(&"function"), "Should include function kind");
        assert!(kinds.contains(&"struct"), "Should include struct kind");
        assert!(kinds.contains(&"enum"), "Should include enum kind");
        assert!(kinds.contains(&"trait"), "Should include trait kind");
        assert!(kinds.contains(&"const"), "Should include const kind");
        assert!(kinds.contains(&"type"), "Should include type kind");
        assert!(kinds.contains(&"module"), "Should include module kind");
    }

    #[test]
    fn test_rust_deps() {
        let source = r#"
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::engine::EngineState;
"#;
        let deps = extract_deps(source, "rs");
        assert_eq!(deps.len(), 3, "Should find 3 use declarations");
        assert_eq!(deps[0].target, "std::collections::HashMap");
        assert_eq!(deps[1].target, "serde::{Serialize, Deserialize}");
        assert_eq!(deps[2].target, "crate::engine::EngineState");
    }

    // ─── TypeScript ────────────────────────────────────────────

    #[test]
    fn test_typescript_symbols() {
        let source = r#"
function greet(name: string): string {
    return `Hello, ${name}`;
}

class Person {
    name: string;
    constructor(name: string) {
        this.name = name;
    }
    sayHello(): void {}
}

interface Talker {
    talk(): void;
}

type Point = {
    x: number;
    y: number;
};

enum Color {
    Red,
    Green,
    Blue,
}
"#;
        let syms = extract_symbols(source, "test.ts", "ts");
        assert!(!syms.is_empty(), "Should find symbols in TS source");

        let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"greet"), "Should find function greet");
        assert!(names.contains(&"Person"), "Should find class Person");
        assert!(names.contains(&"Talker"), "Should find interface Talker");
        assert!(names.contains(&"Point"), "Should find type Point");
        assert!(names.contains(&"Color"), "Should find enum Color");
        assert!(names.contains(&"sayHello"), "Should find method sayHello");
    }

    #[test]
    fn test_typescript_deps() {
        let source = r#"
import { Component } from '@angular/core';
import * as fs from 'fs';
import('./lazy').then(m => m.run());
"#;
        let deps = extract_deps(source, "ts");
        assert!(deps.len() >= 2, "Should find import declarations");
        assert!(deps.iter().any(|d| d.target.contains("@angular/core")));
        assert!(deps.iter().any(|d| d.target.contains("fs")));
    }

    // ─── Python ────────────────────────────────────────────────

    #[test]
    fn test_python_symbols() {
        let source = r#"
def hello(name: str) -> str:
    return f"Hello, {name}"

class Person:
    def __init__(self, name: str):
        self.name = name

    def greet(self) -> str:
        return f"Hi, {self.name}"

@dataclass
class Config:
    debug: bool = False
"#;
        let syms = extract_symbols(source, "test.py", "py");
        assert!(!syms.is_empty(), "Should find symbols in Python source");

        let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"hello"), "Should find function hello");
        assert!(names.contains(&"Person"), "Should find class Person");
        assert!(names.contains(&"Config"), "Should find class Config");
    }

    #[test]
    fn test_python_deps() {
        let source = r#"
import os
import sys
from datetime import datetime
"#;
        let deps = extract_deps(source, "py");
        assert_eq!(deps.len(), 3, "Should find 3 import declarations");
        assert!(deps.iter().any(|d| d.target == "os"));
        assert!(deps.iter().any(|d| d.target == "datetime"));
    }

    // ─── Go ────────────────────────────────────────────────────

    #[test]
    fn test_go_symbols() {
        let source = r#"
package main

func hello(name string) string {
    return "Hello, " + name
}

type User struct {
    Name string
    Age  int
}

type Reader interface {
    Read(p []byte) (n int, err error)
}

func (u *User) Greet() string {
    return "Hi, " + u.Name
}
"#;
        let syms = extract_symbols(source, "test.go", "go");
        assert!(!syms.is_empty(), "Should find symbols in Go source");

        let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"hello"), "Should find function hello");
        assert!(names.contains(&"User"), "Should find type User");
        assert!(names.contains(&"Reader"), "Should find type Reader (interface)");
        assert!(names.contains(&"Greet"), "Should find method Greet");
    }

    #[test]
    fn test_go_deps() {
        let source = r#"
import (
    "fmt"
    "net/http"
    "github.com/gorilla/mux"
)
"#;
        let deps = extract_deps(source, "go");
        assert_eq!(deps.len(), 3, "Should find 3 import declarations");
        assert!(deps.iter().any(|d| d.target == "\"fmt\""));
    }

    // ─── HTML ──────────────────────────────────────────────────

    #[test]
    fn test_html_structure() {
        let source = r#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body>
    <div class="container">
        <h1>Hello</h1>
        <p>World</p>
        <br />
    </div>
    <script>console.log("hi")</script>
</body>
</html>"#;
        let syms = extract_symbols(source, "test.html", "html");
        assert!(!syms.is_empty(), "Should find symbols in HTML source");
        let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"html"), "Should find html element");
        assert!(names.contains(&"div"), "Should find div element");
        assert!(names.contains(&"h1"), "Should find h1 element");
    }

    // ─── Svelte ────────────────────────────────────────────────

    #[test]
    fn test_svelte_structure() {
        let source = r#"<script>
    let count = 0;
    function increment() {
        count += 1;
    }
</script>

<main>
    <h1>Hello Svelte</h1>
    <button on:click={increment}>
        Clicked {count} times
    </button>
</main>

<style>
    h1 { color: red; }
</style>"#;
        let syms = extract_symbols(source, "test.svelte", "svelte");
        assert!(!syms.is_empty(), "Should find symbols in Svelte source");
        let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"main"), "Should find main element");
        assert!(names.contains(&"h1"), "Should find h1 element");
        assert!(names.contains(&"button"), "Should find button element");
    }

    // ─── Language detection ────────────────────────────────────

    #[test]
    fn test_language_detection() {
        assert_eq!(infer_language_from_ext("rs"), Some("rust"));
        assert_eq!(infer_language_from_ext("ts"), Some("typescript"));
        assert_eq!(infer_language_from_ext("tsx"), Some("tsx"));
        assert_eq!(infer_language_from_ext("py"), Some("python"));
        assert_eq!(infer_language_from_ext("go"), Some("go"));
        assert_eq!(infer_language_from_ext("html"), Some("html"));
        assert_eq!(infer_language_from_ext("htm"), Some("html"));
        assert_eq!(infer_language_from_ext("svelte"), Some("svelte"));
        assert_eq!(infer_language_from_ext("js"), None);
        assert_eq!(infer_language_from_ext("css"), None);
    }

    #[test]
    fn test_engine_supported_extensions() {
        let engine = CodeIntelEngine::global();
        let exts = engine.supported_extensions();
        assert!(exts.contains(&"rs"));
        assert!(exts.contains(&"ts"));
        assert!(exts.contains(&"py"));
        assert!(exts.contains(&"go"));
        assert!(exts.contains(&"html"));
        assert!(exts.contains(&"svelte"));
    }

    #[test]
    fn test_unsupported_extension_returns_empty() {
        let syms = extract_symbols("some content", "test.js", "js");
        assert!(syms.is_empty());

        let deps = extract_deps("some content", "js");
        assert!(deps.is_empty());
    }
}
