use std::collections::HashMap;
use std::sync::OnceLock;

use crate::config_types::LspLanguageSettings;

use crate::models::dep_model::CodeIntelDep;
use crate::models::symbol_model::CodeIntelSymbol;
use crate::models::language_model::SupportedLanguage;
use crate::helpers::parser_helper::parse_source;
use crate::helpers::symbols_helper as symbols;

pub(crate) static ENGINE: OnceLock<CodeIntelEngine> = OnceLock::new();
pub(crate) static LSP_CONFIG: OnceLock<HashMap<String, LspLanguageSettings>> = OnceLock::new();

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

    pub fn lsp_command_for(&self, language: &str) -> Option<&LspLanguageSettings> {
        LSP_CONFIG.get().and_then(|m| m.get(language))
    }

    pub fn has_lsp_config(&self) -> bool {
        LSP_CONFIG.get().map_or(false, |m| !m.is_empty())
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
        SupportedLanguage::TypeScript | SupportedLanguage::Tsx => {
            symbols::for_typescript(source, file, &language_name, matches!(lang, SupportedLanguage::Tsx))
        }
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
