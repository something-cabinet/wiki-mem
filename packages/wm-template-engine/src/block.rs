use crate::TemplateError;

pub fn extract_block(remaining: &mut &str, tag: &str) -> Result<String, TemplateError> {
    let mut depth = 1;
    let mut pos = 0;
    let bytes = remaining.as_bytes();

    while pos < remaining.len() {
        if bytes[pos..].starts_with(b"{{") {
            let tag_end = match remaining[pos + 2..].find("}}") {
                Some(e) => pos + 2 + e,
                None => {
                    pos += 2;
                    continue;
                }
            };
            let inner_tag = &remaining[pos + 2..tag_end].trim();

            if let Some(close) = inner_tag.strip_prefix('/') {
                if close == tag {
                    depth -= 1;
                    if depth == 0 {
                        let block_content = remaining[..pos].to_string();
                        *remaining = &remaining[tag_end + 2..];
                        return Ok(block_content);
                    }
                }
            } else if inner_tag.starts_with(&format!("#{}", tag)) {
                depth += 1;
            }
            pos = tag_end + 2;
        } else {
            pos += 1;
        }
    }

    Err(TemplateError::internal(format!(
        "Unclosed block '{}' — missing {{/{}}}",
        tag, tag
    )))
}
