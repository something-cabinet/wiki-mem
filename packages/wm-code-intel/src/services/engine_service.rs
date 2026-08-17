use std::collections::HashMap;
use std::sync::OnceLock;

use crate::config_types::LspLanguageSettings;

use crate::helpers::parser_helper::parse_source;
use crate::helpers::symbols_helper as symbols;
use crate::models::code_edge_model::CodeEdge;
use crate::models::dep_model::CodeIntelDep;
use crate::models::language_model::SupportedLanguage;
use crate::models::symbol_model::CodeIntelSymbol;
use wm_engine::models::edge_type_model::EdgeProvenance;

pub(crate) static ENGINE: OnceLock<CodeIntelEngine> = OnceLock::new();
pub(crate) static LSP_CONFIG: OnceLock<HashMap<String, LspLanguageSettings>> = OnceLock::new();

pub struct CodeIntelEngine {
    languages: HashMap<&'static str, SupportedLanguage>,
}

impl CodeIntelEngine {
    pub fn global() -> &'static CodeIntelEngine {
        ENGINE.get_or_init(CodeIntelEngine::new)
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

    pub fn lsp_command_for(&self, language: &str) -> Option<&LspLanguageSettings> {
        LSP_CONFIG.get().and_then(|m| m.get(language))
    }

    pub fn has_lsp_config(&self) -> bool {
        LSP_CONFIG.get().is_some_and(|m| !m.is_empty())
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

pub fn load_lsp_config(lsp: Option<&HashMap<String, LspLanguageSettings>>) {
    CodeIntelEngine::global().load_lsp_config(lsp);
}

impl CodeIntelEngine {
    pub fn load_lsp_config(&self, lsp: Option<&HashMap<String, LspLanguageSettings>>) {
        if let Some(lsp_map) = lsp {
            if !lsp_map.is_empty() {
                let _ = LSP_CONFIG.set(lsp_map.clone());
            }
        }
    }
}

pub fn extract_symbols(source: &str, file: &str, ext: &str) -> Vec<CodeIntelSymbol> {
    let lang = match SupportedLanguage::from_ext(ext) {
        Some(l) => l,
        None => return Vec::new(),
    };

    let language_name = lang.name().to_string();

    match lang {
        SupportedLanguage::Rust => symbols::for_rust(source, file, &language_name),
        SupportedLanguage::TypeScript | SupportedLanguage::Tsx => symbols::for_typescript(
            source,
            file,
            &language_name,
            matches!(lang, SupportedLanguage::Tsx),
        ),
        SupportedLanguage::Python => symbols::for_python(source, file, &language_name),
        SupportedLanguage::Go => symbols::for_go(source, file, &language_name),
        SupportedLanguage::Html => symbols::for_html(source, file, &language_name),
        SupportedLanguage::Svelte => symbols::for_svelte(source, file, &language_name),
    }
}

pub fn infer_language_from_ext(ext: &str) -> Option<&'static str> {
    SupportedLanguage::from_ext(ext).map(|l| l.name())
}

pub fn extract_deps(source: &str, ext: &str) -> Vec<CodeIntelDep> {
    use streaming_iterator::StreamingIterator;
    use tree_sitter::{Query, QueryCursor};

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
        SupportedLanguage::Python => {
            r"[
            (import_statement name: (dotted_name) @target)
            (import_from_statement module_name: (dotted_name) @target)
        ]"
        }
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
                let mut target = capture
                    .node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("")
                    .to_string();
                if let SupportedLanguage::Rust = lang {
                    if let Some(rest) = target.strip_prefix("use ") {
                        target = rest.to_string();
                    }
                    if target.ends_with(';') {
                        target = target[..target.len().wrapping_sub(1)].to_string();
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
                        line: range.start_point.row.wrapping_add(1),
                        kind: dep_kind.to_string(),
                    });
                }
            }
        }
    }

    results
}

/// Extract typed cross-file code edges from a single
/// file: `imports`, `calls`, `inherits` — raw, per-file facts.
///
/// Deterministic and local: only the given file's source is read.
/// Targets are resolved against the global symbol index at query time
/// (`services::graph_resolver`); this function only captures what the AST
/// directly shows, computing path-math candidates for imports and recording
/// callee/base names + enclosing symbols for calls/inherits. Provenance is
/// `Explicit` here (direct AST reference); the resolver refines it to
/// `Derived`/`Ambiguous` based on symbol resolution.
///
/// `file` is the project-relative path used for import path math.
pub fn extract_edges(source: &str, file: &str, ext: &str) -> Vec<CodeEdge> {
    let lang = match SupportedLanguage::from_ext(ext) {
        Some(l) => l,
        None => return Vec::new(),
    };

    let Ok(tree) = parse_source(source, ext) else {
        return Vec::new();
    };

    let mut edges: Vec<CodeEdge> = Vec::new();

    extract_import_edges(&mut edges, &tree, source, file, &lang);
    extract_deferred_import_edges(&mut edges, &tree, source, file, &lang);
    extract_call_edges(&mut edges, &tree, source, file, &lang);
    extract_inherit_edges(&mut edges, &tree, source, file, &lang);
    extract_implements_edges(&mut edges, &tree, source, file, &lang);
    extract_reference_type_edges(&mut edges, &tree, source, file, &lang);

    edges
}

fn extract_import_edges(
    edges: &mut Vec<CodeEdge>,
    tree: &tree_sitter::Tree,
    source: &str,
    file: &str,
    lang: &SupportedLanguage,
) {
    use streaming_iterator::StreamingIterator;
    use tree_sitter::{Query, QueryCursor};

    let import_query = match lang {
        SupportedLanguage::Rust => r"(use_declaration argument: (_) @target)",
        SupportedLanguage::TypeScript | SupportedLanguage::Tsx => {
            r#"[(import_statement (string (string_fragment) @target))]"#
        }
        SupportedLanguage::Python => {
            r"[
                (import_statement name: (dotted_name) @target)
                (import_from_statement module_name: (dotted_name) @target)
            ]"
        }
        SupportedLanguage::Go => r"(import_spec path: (interpreted_string_literal) @target)",
        SupportedLanguage::Html | SupportedLanguage::Svelte => return,
    };

    let ts_lang = lang.load_language();
    if let Ok(query) = Query::new(&ts_lang, import_query) {
        if let Some(target_index) = query.capture_index_for_name("target") {
            let mut cursor = QueryCursor::new();
            let mut query_matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
            while let Some(match_) = query_matches.next() {
                for capture in match_.captures {
                    if capture.index == target_index {
                        let range = capture.node.range();
                        let raw = capture
                            .node
                            .utf8_text(source.as_bytes())
                            .unwrap_or("")
                            .to_string();
                        let canonical = normalize_import_target(lang, &raw);
                        if canonical.is_empty() {
                            continue;
                        }
                        let candidates = resolve_import_candidates(file, &canonical, lang);
                        let path_math_hint = candidates
                            .as_ref()
                            .and_then(|c| c.first().cloned())
                            .unwrap_or_default();
                        edges.push(CodeEdge {
                            receiver: None,
                            edge_type: "imports".to_string(),
                            source_file: file.to_string(),
                            source_symbol: None,
                            target_file: path_math_hint,
                            target_symbol: Some(canonical),
                            line: range.start_point.row.wrapping_add(1),
                            provenance: EdgeProvenance::Explicit,
                        });
                    }
                }
            }
        }
    }
}

fn extract_deferred_import_edges(
    edges: &mut Vec<CodeEdge>,
    tree: &tree_sitter::Tree,
    source: &str,
    file: &str,
    lang: &SupportedLanguage,
) {
    use streaming_iterator::StreamingIterator;
    use tree_sitter::{Query, QueryCursor};

    let dynamic_import_query = match lang {
        SupportedLanguage::TypeScript | SupportedLanguage::Tsx => {
            r#"(call_expression function: (import) arguments: (arguments (string (string_fragment) @target)))"#
        }
        _ => return,
    };

    let ts_lang = lang.load_language();
    if let Ok(query) = Query::new(&ts_lang, dynamic_import_query) {
        if let Some(target_index) = query.capture_index_for_name("target") {
            let mut cursor = QueryCursor::new();
            let mut query_matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
            while let Some(match_) = query_matches.next() {
                for capture in match_.captures {
                    if capture.index == target_index {
                        let range = capture.node.range();
                        let raw = capture
                            .node
                            .utf8_text(source.as_bytes())
                            .unwrap_or("")
                            .to_string();
                        let canonical = normalize_import_target(lang, &raw);
                        if canonical.is_empty() {
                            continue;
                        }
                        let candidates = resolve_import_candidates(file, &canonical, lang);
                        let path_math_hint = candidates
                            .as_ref()
                            .and_then(|c| c.first().cloned())
                            .unwrap_or_default();
                        edges.push(CodeEdge {
                            receiver: None,
                            edge_type: "imports_deferred".to_string(),
                            source_file: file.to_string(),
                            source_symbol: None,
                            target_file: path_math_hint,
                            target_symbol: Some(canonical),
                            line: range.start_point.row.wrapping_add(1),
                            provenance: EdgeProvenance::Explicit,
                        });
                    }
                }
            }
        }
    }
}

fn extract_call_edges(
    edges: &mut Vec<CodeEdge>,
    tree: &tree_sitter::Tree,
    source: &str,
    file: &str,
    lang: &SupportedLanguage,
) {
    use streaming_iterator::StreamingIterator;
    use tree_sitter::{Query, QueryCursor};

    let call_query = match lang {
        SupportedLanguage::Rust => r#"[
            (call_expression function: (identifier) @name)
            (call_expression function: (field_expression value: (_) @recv field: (field_identifier) @name))
            (call_expression function: (scoped_identifier path: (_) @recv name: (identifier) @name))
        ]"#,
        SupportedLanguage::TypeScript | SupportedLanguage::Tsx => r#"[
            (call_expression function: (identifier) @name)
            (call_expression function: (member_expression object: (_) @recv property: (property_identifier) @name))
        ]"#,
        SupportedLanguage::Python => r#"[
            (call function: (identifier) @name)
            (call function: (attribute object: (_) @recv attribute: (identifier) @name))
        ]"#,
        SupportedLanguage::Go => r#"[
            (call_expression function: (identifier) @name)
            (call_expression function: (selector_expression operand: (_) @recv field: (field_identifier) @name))
        ]"#,
        SupportedLanguage::Html | SupportedLanguage::Svelte => return,
    };

    let ts_lang = lang.load_language();
    if let Ok(query) = Query::new(&ts_lang, call_query) {
        let name_index = query.capture_index_for_name("name");
        let recv_index = query.capture_index_for_name("recv");
        if let Some(ni) = name_index {
            let mut cursor = QueryCursor::new();
            let mut query_matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
            while let Some(match_) = query_matches.next() {
                let mut callee_node: Option<tree_sitter::Node> = None;
                let mut recv_node: Option<tree_sitter::Node> = None;
                for capture in match_.captures {
                    if capture.index == ni {
                        callee_node = Some(capture.node);
                    }
                    if Some(capture.index) == recv_index {
                        recv_node = Some(capture.node);
                    }
                }
                let Some(cn) = callee_node else { continue };
                let callee = cn.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                if callee.is_empty() {
                    continue;
                }
                let receiver = recv_node.and_then(|n| {
                    let text = n.utf8_text(source.as_bytes()).ok()?.to_string();
                    if text.is_empty() {
                        return None;
                    }
                    Some(text)
                });
                let range = cn.range();
                let caller = enclosing_symbol(&cn, source, lang);
                edges.push(CodeEdge {
                    edge_type: "calls".to_string(),
                    source_file: file.to_string(),
                    source_symbol: caller,
                    target_file: String::new(),
                    target_symbol: Some(callee),
                    receiver,
                    line: range.start_point.row.wrapping_add(1),
                    provenance: EdgeProvenance::Explicit,
                });
            }
        }
    }
}

fn extract_inherit_edges(
    edges: &mut Vec<CodeEdge>,
    tree: &tree_sitter::Tree,
    source: &str,
    file: &str,
    lang: &SupportedLanguage,
) {
    use streaming_iterator::StreamingIterator;
    use tree_sitter::{Node, Query, QueryCursor};

    let inherit_query = match lang {
        SupportedLanguage::Rust => {
            r"(trait_item name: (type_identifier) @name bounds: (trait_bounds (type_identifier) @base))"
        }
        SupportedLanguage::TypeScript | SupportedLanguage::Tsx => {
            r"(class_declaration name: (type_identifier) @name (class_heritage (extends_clause value: (identifier) @base)))"
        }
        SupportedLanguage::Python => {
            r"(class_definition name: (identifier) @name superclasses: (argument_list (identifier) @base))"
        }
        SupportedLanguage::Go | SupportedLanguage::Html | SupportedLanguage::Svelte => return,
    };

    let ts_lang = lang.load_language();
    if let Ok(query) = Query::new(&ts_lang, inherit_query) {
        let name_index = query.capture_index_for_name("name");
        let base_index = query.capture_index_for_name("base");
        if let (Some(ni), Some(bi)) = (name_index, base_index) {
            let mut cursor = QueryCursor::new();
            let mut query_matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
            while let Some(match_) = query_matches.next() {
                let mut name_node: Option<Node> = None;
                let mut base_node: Option<Node> = None;
                for capture in match_.captures {
                    if capture.index == ni {
                        name_node = Some(capture.node);
                    } else if capture.index == bi {
                        base_node = Some(capture.node);
                    }
                }
                if let (Some(nn), Some(bn)) = (name_node, base_node) {
                    let name = nn.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let base = bn.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    if name.is_empty() || base.is_empty() {
                        continue;
                    }
                    let range = bn.range();
                    edges.push(CodeEdge {
                        edge_type: "inherits".to_string(),
                        source_file: file.to_string(),
                        source_symbol: Some(name),
                        target_file: String::new(),
                        target_symbol: Some(base),
                        receiver: None,
                        line: range.start_point.row.wrapping_add(1),
                        provenance: EdgeProvenance::Explicit,
                    });
                }
            }
        }
    }
}

fn extract_implements_edges(
    edges: &mut Vec<CodeEdge>,
    tree: &tree_sitter::Tree,
    source: &str,
    file: &str,
    lang: &SupportedLanguage,
) {
    use streaming_iterator::StreamingIterator;
    use tree_sitter::{Node, Query, QueryCursor};

    let implements_query = match lang {
        SupportedLanguage::Rust => {
            r"(impl_item trait: (type_identifier) @base type: (type_identifier) @name)"
        }
        SupportedLanguage::TypeScript | SupportedLanguage::Tsx => {
            r"(class_declaration name: (type_identifier) @name (class_heritage (implements_clause (type_identifier) @base)))"
        }
        _ => return,
    };

    let ts_lang = lang.load_language();
    if let Ok(query) = Query::new(&ts_lang, implements_query) {
        let name_index = query.capture_index_for_name("name");
        let base_index = query.capture_index_for_name("base");
        if let (Some(ni), Some(bi)) = (name_index, base_index) {
            let mut cursor = QueryCursor::new();
            let mut query_matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
            while let Some(match_) = query_matches.next() {
                let mut name_node: Option<Node> = None;
                let mut base_node: Option<Node> = None;
                for capture in match_.captures {
                    if capture.index == ni {
                        name_node = Some(capture.node);
                    } else if capture.index == bi {
                        base_node = Some(capture.node);
                    }
                }
                if let (Some(nn), Some(bn)) = (name_node, base_node) {
                    let name = nn.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let base = bn.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    if name.is_empty() || base.is_empty() {
                        continue;
                    }
                    let range = bn.range();
                    edges.push(CodeEdge {
                        edge_type: "implements".to_string(),
                        source_file: file.to_string(),
                        source_symbol: Some(name),
                        target_file: String::new(),
                        target_symbol: Some(base),
                        receiver: None,
                        line: range.start_point.row.wrapping_add(1),
                        provenance: EdgeProvenance::Explicit,
                    });
                }
            }
        }
    }
}

fn extract_reference_type_edges(
    edges: &mut Vec<CodeEdge>,
    tree: &tree_sitter::Tree,
    source: &str,
    file: &str,
    lang: &SupportedLanguage,
) {
    match lang {
        SupportedLanguage::Rust => {
            let field_query = r#"(field_declaration name: (field_identifier) @_field type: (type_identifier) @type)"#;
            extract_reference_edges(edges, tree, source, file, lang, field_query, "field");

            let param_query = r#"(parameter pattern: (identifier) @_param type: (type_identifier) @type)"#;
            extract_reference_edges(edges, tree, source, file, lang, param_query, "parameter_type");

            let ret_query = r#"(function_item return_type: (type_identifier) @type)"#;
            extract_reference_edges(edges, tree, source, file, lang, ret_query, "return_type");

            let generic_query = r#"(type_arguments (type_identifier) @type)"#;
            extract_reference_edges(edges, tree, source, file, lang, generic_query, "generic_arg");
        }
        SupportedLanguage::TypeScript | SupportedLanguage::Tsx => {
            let field_query = r#"(public_field_definition name: (property_identifier) @_field type: (type_annotation (type_identifier) @type))"#;
            extract_reference_edges(edges, tree, source, file, lang, field_query, "field");

            let param_query = r#"(required_parameter pattern: (identifier) @_param type: (type_annotation (type_identifier) @type))"#;
            extract_reference_edges(edges, tree, source, file, lang, param_query, "parameter_type");

            let ret_query = r#"(function_declaration return_type: (type_annotation (type_identifier) @type))"#;
            extract_reference_edges(edges, tree, source, file, lang, ret_query, "return_type");

            let generic_query = r#"(type_arguments (type_identifier) @type)"#;
            extract_reference_edges(edges, tree, source, file, lang, generic_query, "generic_arg");
        }
        _ => {}
    }
}

/// Helper: extract `references` edges by running a tree-sitter query and
/// emitting an edge for each matched `@type` capture.
fn extract_reference_edges(
    edges: &mut Vec<CodeEdge>,
    tree: &tree_sitter::Tree,
    source: &str,
    file: &str,
    lang: &SupportedLanguage,
    query_str: &str,
    context: &str,
) {
    use streaming_iterator::StreamingIterator;
    use tree_sitter::{Query, QueryCursor};
    let ts_lang = lang.load_language();
    let query = match Query::new(&ts_lang, query_str) {
        Ok(q) => q,
        Err(_) => return,
    };
    let type_index = match query.capture_index_for_name("type") {
        Some(i) => i,
        None => return,
    };
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
    while let Some(m) = matches.next() {
        for capture in m.captures {
            if capture.index == type_index {
                let type_name = capture.node.utf8_text(source.as_bytes()).unwrap_or("");
                if type_name.is_empty() {
                    continue;
                }
                if is_primitive_type(type_name) {
                    continue;
                }
                let range = capture.node.range();
                edges.push(CodeEdge {
                    edge_type: "references".to_string(),
                    source_file: file.to_string(),
                    source_symbol: Some(context.to_string()),
                    target_file: String::new(),
                    target_symbol: Some(type_name.to_string()),
                    receiver: None,
                    line: range.start_point.row.wrapping_add(1),
                    provenance: EdgeProvenance::Explicit,
                });
            }
        }
    }
}

/// Check if a type name is a language primitive that should not generate a reference edge.
fn is_primitive_type(name: &str) -> bool {
    matches!(
        name,
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
            | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
            | "f32" | "f64"
            | "bool" | "char" | "str"
            | "String" | "Vec" | "Option" | "Result" | "Box" | "Arc" | "Rc"
            | "HashMap" | "HashSet" | "BTreeMap" | "BTreeSet"
            | "string" | "number" | "boolean" | "void" | "any" | "unknown" | "never" | "undefined" | "null"
            | "int" | "float" | "dict" | "list" | "tuple" | "set" | "None"
    )
}
/// `as` aliases and brace groups so path math sees a clean module path).
fn normalize_import_target(lang: &SupportedLanguage, raw: &str) -> String {
    let mut t = raw.to_string();
    if matches!(lang, SupportedLanguage::Rust) {
        if let Some(rest) = t.strip_prefix("use ") {
            t = rest.to_string();
        }
        if t.ends_with(';') {
            t = t[..t.len().wrapping_sub(1)].to_string();
        }
        if let Some(idx) = t.find(" as ") {
            t = t[..idx].to_string();
        }
        if let Some(idx) = t.find('{') {
            t = t[..idx].to_string();
            while t.ends_with(':') {
                t.pop();
            }
        }
    }
    if matches!(
        lang,
        SupportedLanguage::TypeScript | SupportedLanguage::Tsx | SupportedLanguage::Go
    ) {
        t = t.trim_matches('"').to_string();
    }
    t
}

/// Path-math candidates for an import target.
///
/// Returns `None` when the import is external/unresolvable, `Some(paths)` with
/// one or more candidate relative file paths otherwise. The resolver confirms
/// index membership and promotes multi-candidate cases to `Ambiguous`.
pub(crate) fn resolve_import_candidates(
    source_file: &str,
    target: &str,
    lang: &SupportedLanguage,
) -> Option<Vec<String>> {
    let source_dir = source_file
        .rsplit_once('/')
        .map(|(d, _)| d.to_string())
        .unwrap_or_default();

    match lang {
        SupportedLanguage::Rust => resolve_rust_import(source_file, target),
        SupportedLanguage::TypeScript | SupportedLanguage::Tsx => {
            resolve_ts_import(&source_dir, target)
        }
        SupportedLanguage::Python => resolve_python_import(&source_dir, target),
        SupportedLanguage::Go => None,
        SupportedLanguage::Html | SupportedLanguage::Svelte => None,
    }
}

fn rust_module_path(target: &str, source_file: &str) -> (String, bool) {
    let mut segments: Vec<&str> = target.split("::").collect();
    if segments.is_empty() {
        return (String::new(), false);
    }
    let mut relative_to_source = false;
    let mut parent_ups = 0usize;
    match segments[0] {
        "crate" => {
            segments.remove(0);
        }
        "self" => {
            segments.remove(0);
            relative_to_source = true;
        }
        "super" => {
            segments.remove(0);
            relative_to_source = true;
            parent_ups = 1;
            while segments.first().is_some_and(|s| *s == "super") {
                parent_ups += 1;
                segments.remove(0);
            }
        }
        _ => {}
    }
    while segments.len() > 1
        && segments
            .last()
            .is_some_and(|s| s.starts_with(char::is_uppercase))
    {
        segments.pop();
    }
    let mut rel = segments.join("/");
    if relative_to_source {
        let mut parts: Vec<&str> = source_file
            .rsplit_once('/')
            .map(|(d, _)| d)
            .unwrap_or("")
            .split('/')
            .collect();
        for _ in 0..parent_ups {
            parts.pop();
        }
        rel = if parts.is_empty() {
            rel
        } else {
            format!("{}/{}", parts.join("/"), rel)
        };
    }
    (rel, relative_to_source)
}

fn resolve_rust_import(source_file: &str, target: &str) -> Option<Vec<String>> {
    let (module_path, _rel) = rust_module_path(target, source_file);
    if module_path.is_empty() {
        return None;
    }
    let first = module_path.split('/').next().unwrap_or("");
    if matches!(first, "std" | "core" | "alloc") {
        return None;
    }
    let mut candidates = Vec::new();
    candidates.push(format!("{}.rs", module_path));
    candidates.push(format!("{}/mod.rs", module_path));
    Some(candidates)
}

fn resolve_ts_import(source_dir: &str, target: &str) -> Option<Vec<String>> {
    if target.starts_with('.') {
        let rel = target.strip_prefix("./").unwrap_or(target);
        let base = join_rel(source_dir, rel);
        Some(ts_file_candidates(&base))
    } else {
        Some(ts_file_candidates(target))
    }
}

fn ts_file_candidates(base: &str) -> Vec<String> {
    let base = base.trim_end_matches('/');
    let mut candidates = Vec::new();
    candidates.push(format!("{}.ts", base));
    candidates.push(format!("{}.tsx", base));
    candidates.push(format!("{}.js", base));
    candidates.push(format!("{}/index.ts", base));
    candidates.push(format!("{}/index.tsx", base));
    candidates
}

fn resolve_python_import(source_dir: &str, target: &str) -> Option<Vec<String>> {
    if target.starts_with('.') {
        let rel = target.trim_start_matches('.');
        let base = join_rel(source_dir, rel);
        Some(python_file_candidates(&base))
    } else {
        let module = target.replace('.', "/");
        Some(python_file_candidates(&module))
    }
}

fn python_file_candidates(base: &str) -> Vec<String> {
    let base = base.trim_end_matches('/');
    let mut candidates = Vec::new();
    candidates.push(format!("{}.py", base));
    candidates.push(format!("{}/__init__.py", base));
    candidates
}

fn join_rel(source_dir: &str, rel: &str) -> String {
    let mut parts: Vec<&str> = source_dir.split('/').collect();
    if source_dir.is_empty() {
        parts.clear();
    }
    for seg in rel.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    parts.join("/")
}

/// Name of the function/method enclosing `node` (the caller for call edges).
fn enclosing_symbol(
    node: &tree_sitter::Node,
    source: &str,
    lang: &SupportedLanguage,
) -> Option<String> {
    let mut cur = node.parent()?;
    loop {
        let kind = cur.kind();
        let name_field = match lang {
            SupportedLanguage::Rust => match kind {
                "function_item" => Some("name"),
                _ => None,
            },
            SupportedLanguage::TypeScript | SupportedLanguage::Tsx => match kind {
                "function_declaration"
                | "method_definition"
                | "generator_function_declaration"
                | "function_expression"
                | "arrow_function" => Some("name"),
                _ => None,
            },
            SupportedLanguage::Python => match kind {
                "function_definition" => Some("name"),
                _ => None,
            },
            SupportedLanguage::Go => match kind {
                "function_declaration" | "method_declaration" => Some("name"),
                _ => None,
            },
            SupportedLanguage::Html | SupportedLanguage::Svelte => None,
        };
        if let Some(field) = name_field {
            if let Some(name_node) = cur.child_by_field_name(field) {
                let name = name_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("")
                    .to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
        cur = cur.parent()?;
    }
}
