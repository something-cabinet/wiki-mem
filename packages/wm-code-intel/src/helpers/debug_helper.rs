#[allow(dead_code)]
pub(crate) fn dump_node_structure(source: &str, ext: &str, max_depth: usize) -> String {
    let parser_mutex = match crate::helpers::parser_helper::get_or_create_parser(ext) {
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
pub(crate) fn dump_node(node: tree_sitter::Node, source: &str, depth: usize, max_depth: usize) -> String {
    if depth > max_depth {
        return String::new();
    }
    let mut result = String::new();
    let kind = node.kind();
    let start = node.start_position().row + 1;
    let end = node.end_position().row + 1;
    let text = node.utf8_text(source.as_bytes()).unwrap_or("");
    let indent = "  ".repeat(depth);

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
