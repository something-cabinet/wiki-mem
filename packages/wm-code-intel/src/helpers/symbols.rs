use crate::models::types::CodeIntelSymbol;
use crate::models::language::SupportedLanguage;
use crate::helpers::parser::{compile_query, parse_source, run_query, get_line_at_offset};

pub(crate) fn for_rust(source: &str, file: &str, language: &str) -> Vec<CodeIntelSymbol> {
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

pub(crate) fn for_typescript(source: &str, file: &str, language: &str, is_tsx: bool) -> Vec<CodeIntelSymbol> {
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

pub(crate) fn for_python(source: &str, file: &str, language: &str) -> Vec<CodeIntelSymbol> {
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

pub(crate) fn for_go(source: &str, file: &str, language: &str) -> Vec<CodeIntelSymbol> {
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

pub(crate) fn for_html(source: &str, file: &str, language: &str) -> Vec<CodeIntelSymbol> {
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

pub(crate) fn for_svelte(source: &str, file: &str, language: &str) -> Vec<CodeIntelSymbol> {
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
