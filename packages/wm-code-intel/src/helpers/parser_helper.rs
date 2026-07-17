use std::sync::OnceLock;

use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

use crate::models::language_model::SupportedLanguage;

pub(crate) struct CompiledQuery {
    pub query: Query,
    pub name_index: u32,
}

pub(crate) fn get_or_create_parser(ext: &str) -> Option<&'static std::sync::Mutex<Parser>> {
    let lang = SupportedLanguage::from_ext(ext)?;
    let name = lang.name();

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

pub(crate) fn parse_source(source: &str, ext: &str) -> Result<tree_sitter::Tree, String> {
    let parser_mutex = get_or_create_parser(ext)
        .ok_or_else(|| format!("Unsupported extension: {}", ext))?;
    let mut parser = parser_mutex.lock().map_err(|e| format!("Parser lock error: {}", e))?;
    parser.parse(source, None).ok_or_else(|| "Failed to parse source".to_string())
}

pub(crate) fn compile_query(lang: &SupportedLanguage, query_str: &str, kind: &'static str) -> Result<CompiledQuery, String> {
    let ts_lang = lang.load_language();
    let query = Query::new(&ts_lang, query_str)
        .map_err(|e| format!("Failed to compile {} query: {}", kind, e))?;

    let name_index = query.capture_index_for_name("name")
        .ok_or_else(|| format!("Query missing @name capture for {}", kind))?;

    Ok(CompiledQuery { query, name_index })
}

pub(crate) fn run_query<'a>(
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
                let start_line = range.start_point.row + 1;
                let start_col = range.start_point.column;

                if !name.is_empty() {
                    results.push((name, start_line, start_col, range.start_byte, range.end_byte));
                }
            }
        }
    }

    results
}

pub(crate) fn get_line_at_offset(source: &str, offset: usize) -> &str {
    let byte_idx = offset.min(source.len());
    let before = &source[..byte_idx];
    let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let after = &source[line_start..];
    after.lines().next().unwrap_or("")
}
