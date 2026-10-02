use std::io;
use std::path::Path;

use crate::page_repo::{FsPageRepo, PageRepo};

use super::page_record_outcome_model::PageRecordOutcome;
use super::record_migration_plan_model::RecordMigrationPlan;

const FRONTMATTER_DELIMITER: &str = "---";
const CLOSING_DELIMITER: &str = "\n---";
const CLOSING_DELIMITER_LEN: usize = 4;

/// Applies every `WouldConvert` page in `plan`, preserving frontmatter.
///
/// Returns the number of pages written. Frontmatter is sliced from the original
/// file byte-for-byte; only the body after the first block is replaced.
pub fn apply_wiki_record_migration(
    wiki_dir: &Path,
    plan: &RecordMigrationPlan,
) -> io::Result<usize> {
    let repo = FsPageRepo;
    let mut written = 0usize;
    for page in &plan.pages {
        let PageRecordOutcome::WouldConvert { body } = &page.outcome else {
            continue;
        };
        let path = wiki_dir.join(&page.path);
        let content = repo.read_to_string(&path)?;
        let Some(updated) = replace_page_body(&content, body) else {
            continue;
        };
        repo.write(&path, updated.as_bytes())?;
        written = written.saturating_add(1);
    }
    Ok(written)
}

/// Replaces the body after the first frontmatter block, which is preserved.
pub fn replace_page_body(content: &str, new_body: &str) -> Option<String> {
    let prefix = frontmatter_prefix(content)?;
    let mut updated = String::with_capacity(prefix.len().saturating_add(new_body.len()));
    updated.push_str(prefix);
    updated.push('\n');
    updated.push('\n');
    updated.push_str(new_body.trim_end_matches('\n'));
    updated.push('\n');
    Some(updated)
}

fn frontmatter_prefix(content: &str) -> Option<&str> {
    let rest = content.trim_start();
    if !rest.starts_with(FRONTMATTER_DELIMITER) {
        return None;
    }
    let pos = rest[FRONTMATTER_DELIMITER.len()..].find(CLOSING_DELIMITER)?;
    let close = FRONTMATTER_DELIMITER.len().saturating_add(pos);
    Some(&rest[..close.saturating_add(CLOSING_DELIMITER_LEN)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::services::record_migration::{
        plan_record_migration_filtered, RecordMigrationFilter,
    };

    const FRONTMATTER: &str = "---\ntitle: One\nid: \"wiki:decisions:one\"\ntype: decision\n---\n";

    #[test]
    fn preserves_frontmatter_byte_for_byte() {
        let content = format!("{FRONTMATTER}\nold body\n\nmore\n");
        let updated =
            replace_page_body(&content, "schema_version: 1\nstate: |-\n  new\n").expect("replace");

        assert!(updated.starts_with(FRONTMATTER));
        assert!(updated.contains("schema_version: 1"));
        assert!(!updated.contains("old body"));
    }

    #[test]
    fn deduplicates_leading_frontmatter_blocks() {
        let content = format!("{FRONTMATTER}\n---\ntitle: Two\n---\n\nold body\n");
        let updated = replace_page_body(&content, "new body").expect("replace");

        assert_eq!(updated.matches("title:").count(), 1);
        assert!(updated.ends_with("new body\n"));
    }

    #[test]
    fn returns_none_without_frontmatter() {
        assert!(replace_page_body("no frontmatter", "body").is_none());
    }

    #[test]
    fn apply_filtered_plan_touches_only_selected_page() {
        let temp = tempfile::tempdir().expect("tempdir");
        let selected = temp
            .path()
            .join("decisions/one.md")
            .to_string_lossy()
            .to_string();
        let untouched = temp
            .path()
            .join("decisions/two.md")
            .to_string_lossy()
            .to_string();
        std::fs::create_dir_all(temp.path().join("decisions")).expect("dir");
        std::fs::write(&selected, format!("{FRONTMATTER}\n## Context\n\nprose\n")).expect("write");
        std::fs::write(&untouched, format!("{FRONTMATTER}\n## Context\n\nprose\n")).expect("write");
        let filter = RecordMigrationFilter::new(vec!["decisions/one.md".to_owned()], None);
        let plan = plan_record_migration_filtered(temp.path(), &filter);

        let written = apply_wiki_record_migration(temp.path(), &plan).expect("apply");

        assert_eq!(written, 1);
        assert!(std::fs::read_to_string(&selected)
            .expect("selected")
            .contains("schema_version: 1"));
        assert!(!std::fs::read_to_string(&untouched)
            .expect("untouched")
            .contains("schema_version: 1"));
    }
}
