use crate::template_engine::TemplateError;

pub fn extract_block(remaining: &mut &str, tag: &str) -> Result<String, TemplateError> {
    let mut depth = 1usize;
    let mut pos = 0;
    let bytes = remaining.as_bytes();

    while pos < remaining.len() {
        if bytes[pos..].starts_with(b"{{") {
            let tag_end = match remaining[pos.wrapping_add(2)..].find("}}") {
                Some(e) => pos.wrapping_add(2).wrapping_add(e),
                None => {
                    pos = pos.wrapping_add(2);
                    continue;
                }
            };
            let inner_tag = &remaining[pos.wrapping_add(2)..tag_end].trim();

            if let Some(close) = inner_tag.strip_prefix('/') {
                if close == tag {
                    depth = depth.wrapping_sub(1);
                    if depth == 0 {
                        let block_content = remaining[..pos].to_string();
                        *remaining = &remaining[tag_end.wrapping_add(2)..];
                        return Ok(block_content);
                    }
                }
            } else if inner_tag.starts_with(&format!("#{}", tag)) {
                depth = depth.wrapping_add(1);
            }
            pos = tag_end.wrapping_add(2);
        } else {
            pos = pos.wrapping_add(1);
        }
    }

    Err(TemplateError::internal(format!(
        "Unclosed block '{}' — missing {{/{}}}",
        tag, tag
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_block() {
        let mut s = "content{{/if}} trailing";
        let result = extract_block(&mut s, "if").unwrap();
        assert_eq!(result, "content");
        assert_eq!(s, " trailing");
    }

    #[test]
    fn test_nested_block() {
        let mut s = "{{#if b}}deep{{/if}}{{/if}} end";
        let result = extract_block(&mut s, "if").unwrap();
        assert_eq!(result, "{{#if b}}deep{{/if}}");
        assert_eq!(s, " end");
    }

    #[test]
    fn test_unclosed_block_error() {
        let mut s = "{{#if cond}}content";
        let result = extract_block(&mut s, "if");
        assert!(result.is_err());
    }
}
