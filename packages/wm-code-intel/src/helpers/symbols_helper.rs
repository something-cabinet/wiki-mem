use crate::models::symbol_model::CodeIntelSymbol;
use crate::models::language_model::SupportedLanguage;
use crate::helpers::parser_helper::{compile_query, parse_source, run_query, get_line_at_offset};

fn for_language(
    source: &str,
    file: &str,
    language: &str,
    lang: &SupportedLanguage,
    ext: &str,
    queries: &[(&str, &'static str)],
) -> Vec<CodeIntelSymbol> {
    let mut results = Vec::new();
    let Ok(tree) = parse_source(source, ext) else { return results };
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

pub(crate) fn for_rust(source: &str, file: &str, language: &str) -> Vec<CodeIntelSymbol> {
    for_language(source, file, language, &SupportedLanguage::Rust, "rs", &[
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
    ])
}

pub(crate) fn for_typescript(source: &str, file: &str, language: &str, is_tsx: bool) -> Vec<CodeIntelSymbol> {
    let (lang, ext) = if is_tsx {
        (SupportedLanguage::Tsx, "tsx")
    } else {
        (SupportedLanguage::TypeScript, "ts")
    };
    for_language(source, file, language, &lang, ext, &[
        (r"(function_declaration name: (identifier) @name)", "function"),
        (r"(method_definition name: (property_identifier) @name)", "method"),
        (r"(class_declaration name: (type_identifier) @name)", "class"),
        (r"(interface_declaration name: (type_identifier) @name)", "interface"),
        (r"(type_alias_declaration name: (type_identifier) @name)", "type"),
        (r"(enum_declaration name: (identifier) @name)", "enum"),
        (r"(abstract_method_signature name: (property_identifier) @name)", "method"),
        (r"(variable_declarator name: (identifier) @name)", "variable"),
    ])
}

pub(crate) fn for_python(source: &str, file: &str, language: &str) -> Vec<CodeIntelSymbol> {
    for_language(source, file, language, &SupportedLanguage::Python, "py", &[
        (r"(function_definition name: (identifier) @name)", "function"),
        (r"(class_definition name: (identifier) @name)", "class"),
        (r"(decorated_definition (function_definition name: (identifier) @name))", "function"),
        (r"(decorated_definition (class_definition name: (identifier) @name))", "class"),
    ])
}

pub(crate) fn for_go(source: &str, file: &str, language: &str) -> Vec<CodeIntelSymbol> {
    for_language(source, file, language, &SupportedLanguage::Go, "go", &[
        (r"(function_declaration name: (identifier) @name)", "function"),
        (r"(method_declaration name: (field_identifier) @name)", "method"),
        (r"(type_declaration (type_spec name: (type_identifier) @name))", "type"),
    ])
}

pub(crate) fn for_html(source: &str, file: &str, language: &str) -> Vec<CodeIntelSymbol> {
    for_language(source, file, language, &SupportedLanguage::Html, "html", &[
        (r"(element (start_tag (tag_name) @name))", "element"),
        (r"(self_closing_tag (tag_name) @name)", "element"),
    ])
}

pub(crate) fn for_svelte(source: &str, file: &str, language: &str) -> Vec<CodeIntelSymbol> {
    for_language(source, file, language, &SupportedLanguage::Svelte, "svelte", &[
        (r"(element (start_tag (tag_name) @name))", "element"),
        (r"(component (identifier) @name)", "component"),
    ])
}
