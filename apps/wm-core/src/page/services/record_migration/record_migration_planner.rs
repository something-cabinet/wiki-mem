use std::fs;
use std::path::{Path, PathBuf};

use wm_engine::{parse_record, record_scope_for_dir, PageType};

use super::frontmatter_prepass;
use super::migration_note_model::MigrationNote;
use super::page_record_outcome_model::PageRecordOutcome;
use super::page_record_plan_model::PageRecordPlan;
use super::parse_failure_model::ParseFailure;
use super::record_body_builder::build_record_body;
use super::record_migration_filter_model::RecordMigrationFilter;
use super::record_migration_plan_model::RecordMigrationPlan;
use super::skip_reason_model::SkipReason;

const MARKDOWN_EXTENSION: &str = "md";
const INDEX_FILE_NAMES: [&str; 2] = ["index.md", "log.md"];
const INDEX_LABEL: &str = "index";
const SCHEMA_VERSION_MARKER: &str = "schema_version: 1";
const ESCAPED_NEWLINE: &str = "\\n";
const READ_ERROR: &str = "failed to read file";
const RELATED_UNION_NOTE: &str = "duplicate frontmatter: first block wins, relates_to unioned";
const ESCAPED_DECODED_NOTE: &str = "escaped newline body decoded (single-line case)";
const ESCAPED_PRESERVED_NOTE: &str =
    "escaped newline body preserved (ambiguous multi-line case)";

pub fn plan_record_migration(wiki_dir: &Path) -> RecordMigrationPlan {
    plan_record_migration_filtered(wiki_dir, &RecordMigrationFilter::default())
}

pub fn plan_record_migration_filtered(
    wiki_dir: &Path,
    filter: &RecordMigrationFilter,
) -> RecordMigrationPlan {
    let mut plan = RecordMigrationPlan::default();
    let mut files = markdown_files(wiki_dir, filter);
    files.sort();
    for path in files {
        plan.scanned = plan.scanned.saturating_add(1);
        plan_page(wiki_dir, &path, filter, &mut plan);
    }
    plan
}

fn markdown_files(wiki_dir: &Path, filter: &RecordMigrationFilter) -> Vec<PathBuf> {
    walkdir::WalkDir::new(wiki_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map(|extension| extension == MARKDOWN_EXTENSION)
                .unwrap_or(false)
        })
        .filter(|entry| filter.accepts(&relative_wiki_path(wiki_dir, entry.path())))
        .map(|entry| entry.path().to_path_buf())
        .collect()
}

fn plan_page(
    wiki_dir: &Path,
    path: &Path,
    filter: &RecordMigrationFilter,
    plan: &mut RecordMigrationPlan,
) {
    let relative = relative_wiki_path(wiki_dir, path);
    let Ok(content) = fs::read_to_string(path) else {
        plan.parse_failures.push(ParseFailure {
            path: relative,
            error: READ_ERROR.to_owned(),
        });
        return;
    };
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if INDEX_FILE_NAMES.contains(&file_name) {
        plan.count_excluded(INDEX_LABEL);
        return;
    }
    let directory = directory_label(wiki_dir, path);
    let Some(page_type) = record_scope_for_dir(&directory) else {
        plan.count_excluded(&directory);
        return;
    };
    plan_target_page(&relative, &page_type, content, filter, plan);
}

fn plan_target_page(
    relative: &str,
    page_type: &PageType,
    content: String,
    filter: &RecordMigrationFilter,
    plan: &mut RecordMigrationPlan,
) {
    let (blocks, body) = frontmatter_prepass::split_leading_frontmatter(&content);
    if blocks.is_empty() {
        push_skip(plan, relative, page_type, SkipReason::MissingFrontmatter);
        return;
    }
    if let Err(reason) = frontmatter_prepass::check_duplicate_frontmatter(&blocks) {
        push_skip(plan, relative, page_type, reason);
        return;
    }
    if blocks.len() > 1 {
        plan.notes.push(MigrationNote {
            path: relative.to_owned(),
            note: RELATED_UNION_NOTE.to_owned(),
        });
    }
    if is_already_record(&body) {
        plan.already_record = plan.already_record.saturating_add(1);
        plan.pages.push(PageRecordPlan {
            path: relative.to_owned(),
            page_type: page_type.as_str().to_owned(),
            outcome: PageRecordOutcome::AlreadyRecord,
        });
        return;
    }
    let (state, escaped_newlines_decoded, escaped_newlines_preserved) = normalize_state(&body);
    if state.is_empty() {
        push_skip(plan, relative, page_type, SkipReason::EmptyState);
        return;
    }
    if escaped_newlines_decoded {
        plan.notes.push(MigrationNote {
            path: relative.to_owned(),
            note: ESCAPED_DECODED_NOTE.to_owned(),
        });
    }
    if escaped_newlines_preserved {
        plan.notes.push(MigrationNote {
            path: relative.to_owned(),
            note: ESCAPED_PRESERVED_NOTE.to_owned(),
        });
    }
    if filter.limit_reached(plan.would_convert_total()) {
        push_skip(plan, relative, page_type, SkipReason::LimitReached);
        return;
    }
    let body_out = match build_record_body(page_type, &state) {
        Ok(body) => body,
        Err(error) => {
            plan.parse_failures.push(ParseFailure {
                path: relative.to_owned(),
                error: error.clone(),
            });
            push_skip(plan, relative, page_type, SkipReason::InvalidRecordBody { error });
            return;
        }
    };
    if let Err(error) = parse_record(page_type, &body_out) {
        let error = error.to_string();
        plan.parse_failures.push(ParseFailure {
            path: relative.to_owned(),
            error: error.clone(),
        });
        push_skip(plan, relative, page_type, SkipReason::InvalidRecordBody { error });
        return;
    }
    plan.count_convert(page_type.as_str());
    plan.pages.push(PageRecordPlan {
        path: relative.to_owned(),
        page_type: page_type.as_str().to_owned(),
        outcome: PageRecordOutcome::WouldConvert { body: body_out },
    });
}

fn push_skip(
    plan: &mut RecordMigrationPlan,
    relative: &str,
    page_type: &PageType,
    reason: SkipReason,
) {
    plan.count_skip(page_type.as_str(), &reason);
    plan.pages.push(PageRecordPlan {
        path: relative.to_owned(),
        page_type: page_type.as_str().to_owned(),
        outcome: PageRecordOutcome::Skip { reason },
    });
}

fn is_already_record(body: &str) -> bool {
    body.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .is_some_and(|line| line == SCHEMA_VERSION_MARKER)
}

fn normalize_state(body: &str) -> (String, bool, bool) {
    let trimmed = body.trim();
    if !trimmed.contains(ESCAPED_NEWLINE) {
        return (trimmed.to_owned(), false, false);
    }
    if !trimmed.contains('\n') {
        return (trimmed.replace(ESCAPED_NEWLINE, "\n"), true, false);
    }
    (trimmed.to_owned(), false, true)
}

fn relative_wiki_path(wiki_dir: &Path, path: &Path) -> String {
    path.strip_prefix(wiki_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn directory_label(wiki_dir: &Path, path: &Path) -> String {
    path.strip_prefix(wiki_dir)
        .ok()
        .and_then(|relative| relative.components().next())
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::services::record_migration::page_record_outcome_model::PageRecordOutcome;

    fn write_fixture(root: &Path, relative: &str, content: &str) {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().expect("fixture has a parent"))
            .expect("fixture dir should be created");
        std::fs::write(path, content).expect("fixture should be written");
    }

    fn decision_page(title: &str) -> String {
        format!("---\ntitle: {title}\nid: \"wiki:decisions:{title}\"\ntype: decision\n---\n\n## Context\n\nSome prose.\n")
    }

    fn convert_count(plan: &RecordMigrationPlan, page_type: &str) -> usize {
        plan.would_convert.get(page_type).copied().unwrap_or(0)
    }

    #[test]
    fn counts_targets_and_excludes_others() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_fixture(temp.path(), "decisions/one.md", &decision_page("one"));
        write_fixture(temp.path(), "patterns/two.md", &decision_page("two").replace("decision", "pattern"));
        write_fixture(temp.path(), "tasks/three.md", "---\ntype: task\n---\n\ntask\n");
        write_fixture(temp.path(), "learnings/four.md", "---\ntype: concept\n---\n\nlearning\n");
        write_fixture(temp.path(), "index.md", "# Index\n");

        let plan = plan_record_migration(temp.path());

        assert_eq!(plan.scanned, 5);
        assert_eq!(convert_count(&plan, "decision"), 1);
        assert_eq!(convert_count(&plan, "pattern"), 1);
        assert_eq!(convert_count(&plan, "task"), 1);
        assert_eq!(plan.excluded.get("task"), None);
        assert_eq!(plan.excluded.get("learnings"), Some(&1));
        assert_eq!(plan.excluded.get("index"), Some(&1));
    }

    #[test]
    fn converted_body_replans_as_noop() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_fixture(temp.path(), "concepts/one.md", "---\ntype: concept\n---\n\n## Body\n");
        let first = plan_record_migration(temp.path());
        let body = first
            .pages
            .iter()
            .find_map(|page| match &page.outcome {
                PageRecordOutcome::WouldConvert { body } => Some(body.clone()),
                _ => None,
            })
            .expect("page should be convertible");

        write_fixture(
            temp.path(),
            "concepts/one.md",
            &format!("---\ntype: concept\n---\n\n{body}"),
        );
        let second = plan_record_migration(temp.path());

        assert_eq!(second.already_record, 1);
        assert_eq!(convert_count(&second, "concept"), 0);
    }

    #[test]
    fn skips_duplicate_frontmatter_scalar_conflict() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_fixture(
            temp.path(),
            "decisions/dup.md",
            "---\ntitle: One\ntype: decision\n---\n\n---\ntitle: Two\ntype: decision\n---\n\n## Body\n",
        );

        let plan = plan_record_migration(temp.path());

        assert_eq!(convert_count(&plan, "decision"), 0);
        assert_eq!(
            plan.skipped
                .get("decision")
                .and_then(|reasons| reasons.get("duplicate-frontmatter-conflict")),
            Some(&1)
        );
    }

    #[test]
    fn decodes_unambiguous_escaped_newline_body() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_fixture(
            temp.path(),
            "concepts/escaped.md",
            "---\ntype: concept\n---\n\n## Body\\n\\nprose",
        );

        let plan = plan_record_migration(temp.path());

        assert_eq!(convert_count(&plan, "concept"), 1);
        assert!(plan
            .notes
            .iter()
            .any(|note| note.note.contains("decoded")));
    }

    #[test]
    fn skips_empty_body() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_fixture(temp.path(), "howto/empty.md", "---\ntype: howto\n---\n\n");

        let plan = plan_record_migration(temp.path());

        assert_eq!(convert_count(&plan, "howto"), 0);
        assert_eq!(
            plan.skipped
                .get("howto")
                .and_then(|reasons| reasons.get("empty-state")),
            Some(&1)
        );
    }

    #[test]
    fn only_filter_restricts_planned_pages() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_fixture(temp.path(), "decisions/one.md", &decision_page("one"));
        write_fixture(temp.path(), "decisions/two.md", &decision_page("two"));
        let filter = RecordMigrationFilter::new(vec!["decisions/one.md".to_owned()], None);

        let plan = plan_record_migration_filtered(temp.path(), &filter);

        assert_eq!(plan.scanned, 1);
        assert_eq!(plan.pages.len(), 1);
        assert_eq!(plan.pages[0].path, "decisions/one.md");
        assert_eq!(convert_count(&plan, "decision"), 1);
    }

    #[test]
    fn only_filter_reports_selected_page_as_already_record() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_fixture(
            temp.path(),
            "concepts/one.md",
            "---\ntype: concept\n---\n\n## Body\n",
        );
        let filter = RecordMigrationFilter::new(vec!["concepts/one.md".to_owned()], None);
        let first = plan_record_migration_filtered(temp.path(), &filter);
        let body = first
            .pages
            .iter()
            .find_map(|page| match &page.outcome {
                PageRecordOutcome::WouldConvert { body } => Some(body.clone()),
                _ => None,
            })
            .expect("page should be convertible");

        write_fixture(
            temp.path(),
            "concepts/one.md",
            &format!("---\ntype: concept\n---\n\n{body}"),
        );
        let second = plan_record_migration_filtered(temp.path(), &filter);

        assert_eq!(second.already_record, 1);
        assert_eq!(convert_count(&second, "concept"), 0);
    }

    #[test]
    fn limit_caps_conversions() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_fixture(temp.path(), "decisions/one.md", &decision_page("one"));
        write_fixture(temp.path(), "decisions/two.md", &decision_page("two"));
        write_fixture(temp.path(), "decisions/three.md", &decision_page("three"));
        let filter = RecordMigrationFilter::new(Vec::new(), Some(2));

        let plan = plan_record_migration_filtered(temp.path(), &filter);

        assert_eq!(plan.would_convert_total(), 2);
        assert_eq!(
            plan.skipped
                .get("decision")
                .and_then(|reasons| reasons.get("limit-reached")),
            Some(&1)
        );
    }
}
