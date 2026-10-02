use serde_yaml::Value;

use super::skip_reason_model::SkipReason;

const FRONTMATTER_DELIMITER: &str = "---";
const CLOSING_DELIMITER: &str = "\n---";
const CLOSING_DELIMITER_LEN: usize = 4;
const RELATES_TO_KEY: &str = "relates_to";

/// Splits every leading `---`-delimited frontmatter block from the real body.
///
/// Returns the raw YAML of each block (fences stripped) and the remaining body.
/// Multiple leading blocks are the pre-existing duplicated-frontmatter bug.
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

/// Detects an unsafe duplicated-frontmatter page.
///
/// The first block is authoritative and `relates_to` is unioned. Any other
/// top-level value that differs between blocks makes the page unsafe.
pub fn check_duplicate_frontmatter(blocks: &[String]) -> Result<(), SkipReason> {
    let Some((first, rest)) = blocks.split_first() else {
        return Ok(());
    };
    if rest.is_empty() {
        return Ok(());
    }
    let first = parse_mapping(first).ok_or(SkipReason::UnparseableFrontmatter { block: 0 })?;
    for (offset, block) in rest.iter().enumerate() {
        let index = offset.saturating_add(1);
        let other =
            parse_mapping(block).ok_or(SkipReason::UnparseableFrontmatter { block: index })?;
        if let Some(key) = conflicting_key(&first, &other) {
            return Err(SkipReason::DuplicateFrontmatterConflict { key });
        }
    }
    Ok(())
}

fn parse_mapping(block: &str) -> Option<serde_yaml::Mapping> {
    match serde_yaml::from_str::<Value>(block) {
        Ok(Value::Mapping(mapping)) => Some(mapping),
        _ => None,
    }
}

fn conflicting_key(first: &serde_yaml::Mapping, other: &serde_yaml::Mapping) -> Option<String> {
    for (key, value) in other {
        let Some(key_name) = key.as_str() else {
            continue;
        };
        if key_name == RELATES_TO_KEY {
            continue;
        }
        let Some(existing) = first.get(key_name) else {
            continue;
        };
        if existing != value {
            return Some(key_name.to_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const SINGLE: &str = "---\ntitle: One\ntype: concept\n---\n\n## Body\ntext\n";
    const DUPLICATE_SAME: &str =
        "---\ntitle: One\ntype: concept\n---\n\n---\ntitle: One\ntype: concept\n---\n\n## Body\n";
    const DUPLICATE_RELATES: &str =
        "---\ntitle: One\nrelates_to:\n  - a\n---\n\n---\ntitle: One\nrelates_to:\n  - b\n---\n\n## Body\n";
    const DUPLICATE_CONFLICT: &str =
        "---\ntitle: One\ntype: concept\n---\n\n---\ntitle: Two\ntype: concept\n---\n\n## Body\n";

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
    fn accepts_identical_duplicate_blocks() {
        let (blocks, _) = split_leading_frontmatter(DUPLICATE_SAME);
        assert_eq!(check_duplicate_frontmatter(&blocks), Ok(()));
    }

    #[test]
    fn accepts_relates_to_that_differs() {
        let (blocks, _) = split_leading_frontmatter(DUPLICATE_RELATES);
        assert_eq!(check_duplicate_frontmatter(&blocks), Ok(()));
    }

    #[test]
    fn rejects_scalar_conflict_between_blocks() {
        let (blocks, _) = split_leading_frontmatter(DUPLICATE_CONFLICT);
        assert_eq!(
            check_duplicate_frontmatter(&blocks),
            Err(SkipReason::DuplicateFrontmatterConflict {
                key: "title".to_owned()
            })
        );
    }

    #[test]
    fn accepts_content_without_frontmatter() {
        let (blocks, body) = split_leading_frontmatter("no frontmatter here\n");
        assert!(blocks.is_empty());
        assert_eq!(body, "no frontmatter here\n");
    }
}
