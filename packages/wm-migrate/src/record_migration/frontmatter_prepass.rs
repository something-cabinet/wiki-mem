const FRONTMATTER_DELIMITER: &str = "---";
const CLOSING_DELIMITER: &str = "\n---";
const CLOSING_DELIMITER_LEN: usize = 4;

pub fn split_leading_frontmatter(content: &str) -> (Vec<String>, String) {
    let mut blocks = Vec::new();
    let mut rest = content.trim_start();
    while rest.starts_with(FRONTMATTER_DELIMITER) {
        let Some(pos) = rest[FRONTMATTER_DELIMITER.len()..].find(CLOSING_DELIMITER) else {
            break;
        };
        let close = FRONTMATTER_DELIMITER.len().saturating_add(pos);
        let raw = rest
            .get(CLOSING_DELIMITER_LEN..close)
            .unwrap_or_default()
            .to_owned();
        let next = close.saturating_add(CLOSING_DELIMITER_LEN);
        blocks.push(raw);
        rest = rest[next..].trim_start();
    }
    (blocks, rest.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SINGLE: &str = "---\ntitle: One\ntype: concept\n---\n\n## Body\ntext\n";
    const DUPLICATE_SAME: &str =
        "---\ntitle: One\ntype: concept\n---\n\n---\ntitle: One\ntype: concept\n---\n\n## Body\n";

    #[test]
    fn splits_single_block_from_body() {
        let (blocks, body) = split_leading_frontmatter(SINGLE);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0], "title: One\ntype: concept");
        assert_eq!(body, "## Body\ntext\n");
    }

    #[test]
    fn splits_every_leading_block() {
        let (blocks, body) = split_leading_frontmatter(DUPLICATE_SAME);
        assert_eq!(blocks.len(), 2);
        assert_eq!(body, "## Body\n");
    }

    #[test]
    fn accepts_content_without_frontmatter() {
        let (blocks, body) = split_leading_frontmatter("no frontmatter here\n");
        assert!(blocks.is_empty());
        assert_eq!(body, "no frontmatter here\n");
    }
}
