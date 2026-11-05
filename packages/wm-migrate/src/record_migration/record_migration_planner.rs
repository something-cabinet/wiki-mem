use std::fs;
use std::path::{Path, PathBuf};

use wm_engine::{parse_record, record_scope_for_dir, PageType};

use super::frontmatter_merge_helper::merge_frontmatter_blocks;
use super::frontmatter_prepass;
use super::migration_note_model::MigrationNote;
use super::page_content_composer::compose_page;
use super::page_record_outcome_model::PageRecordOutcome;
use super::page_record_plan_model::PageRecordPlan;
use super::page_version_helper::page_version;
use super::parse_failure_model::ParseFailure;
use super::record_body_builder::build_record_body;
use super::record_migration_applier::replace_page_body;
use super::record_migration_filter_model::RecordMigrationFilter;
use super::record_migration_plan_model::RecordMigrationPlan;
use super::skip_reason_model::SkipReason;
use super::version_upgrade_service::{current_record_version, upgrade_record_body, UPGRADE_STEPS};

const MARKDOWN_EXTENSION: &str = "md";
const INDEX_FILE_NAMES: [&str; 2] = ["index.md", "log.md"];
const INDEX_LABEL: &str = "index";
const UNVERSIONED: u32 = 0;
const CRLF: &str = "\r\n";
const LF: &str = "\n";
const STEP_NORMALIZE_LINE_ENDINGS: &str = "normalize-line-endings";
const STEP_REPAIR_FRONTMATTER: &str = "repair-frontmatter";
const STEP_CONVERT_PROSE: &str = "convert-prose";
const STEP_UPGRADE_VERSION: &str = "upgrade-version";
const ESCAPED_NEWLINE: &str = "\\n";
const READ_ERROR: &str = "failed to read file";
const MERGED_FRONTMATTER_NOTE: &str = "frontmatter blocks merged";
const ESCAPED_DECODED_NOTE: &str = "escaped newline body decoded (single-line case)";
const ESCAPED_PRESERVED_NOTE: &str = "escaped newline body preserved (ambiguous multi-line case)";

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
    let normalized = content.replace(CRLF, LF);
    let mut steps: Vec<String> = Vec::new();
    if normalized != content {
        steps.push(STEP_NORMALIZE_LINE_ENDINGS.to_owned());
    }
    let (blocks, body) = frontmatter_prepass::split_leading_frontmatter(&normalized);
    if blocks.is_empty() {
        push_skip(plan, relative, page_type, SkipReason::MissingFrontmatter);
        return;
    }
    let frontmatter = match merge_frontmatter_blocks(&blocks) {
        Ok(frontmatter) => frontmatter,
        Err(reason) => {
            push_skip(plan, relative, page_type, reason);
            return;
        }
    };
    if blocks.len() > 1 {
        steps.push(STEP_REPAIR_FRONTMATTER.to_owned());
        plan.notes.push(MigrationNote {
            path: relative.to_owned(),
            note: MERGED_FRONTMATTER_NOTE.to_owned(),
        });
    }
    let new_body = if page_version(&body) == UNVERSIONED {
        let Some(converted) = convert_prose(relative, page_type, &body, plan) else {
            return;
        };
        steps.push(STEP_CONVERT_PROSE.to_owned());
        converted
    } else {
        match upgrade_record_body(&body, &UPGRADE_STEPS, current_record_version()) {
            Ok((upgraded, 0)) => upgraded,
            Ok((upgraded, _)) => {
                steps.push(STEP_UPGRADE_VERSION.to_owned());
                upgraded
            }
            Err(reason) => {
                push_skip(plan, relative, page_type, reason);
                return;
            }
        }
    };
    if steps.is_empty() {
        plan.already_record = plan.already_record.saturating_add(1);
        plan.pages.push(PageRecordPlan {
            path: relative.to_owned(),
            page_type: page_type.as_str().to_owned(),
            outcome: PageRecordOutcome::AlreadyRecord,
        });
        return;
    }
    if filter.limit_reached(plan.would_convert_total()) {
        push_skip(plan, relative, page_type, SkipReason::LimitReached);
        return;
    }
    let updated = match blocks.len() {
        1 => replace_page_body(&normalized, &new_body).unwrap_or_default(),
        _ => compose_page(&frontmatter, &new_body),
    };
    plan.count_convert(page_type.as_str());
    for step in &steps {
        plan.count_step(step);
    }
    plan.pages.push(PageRecordPlan {
        path: relative.to_owned(),
        page_type: page_type.as_str().to_owned(),
        outcome: PageRecordOutcome::WouldConvert {
            content: updated,
            steps,
        },
    });
}

fn convert_prose(
    relative: &str,
    page_type: &PageType,
    body: &str,
    plan: &mut RecordMigrationPlan,
) -> Option<String> {
    let (state, escaped_newlines_decoded, escaped_newlines_preserved) = normalize_state(body);
    if state.is_empty() {
        push_skip(plan, relative, page_type, SkipReason::EmptyState);
        return None;
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
    let body_out = match build_record_body(page_type, &state) {
        Ok(body) => body,
        Err(error) => {
            record_failure(plan, relative, page_type, error);
            return None;
        }
    };
    if let Err(error) = parse_record(page_type, &body_out) {
        record_failure(plan, relative, page_type, error.to_string());
        return None;
    }
    Some(body_out)
}

fn record_failure(
    plan: &mut RecordMigrationPlan,
    relative: &str,
    page_type: &PageType,
    error: String,
) {
    plan.parse_failures.push(ParseFailure {
        path: relative.to_owned(),
        error: error.clone(),
    });
    push_skip(
        plan,
        relative,
        page_type,
        SkipReason::InvalidRecordBody { error },
    );
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
    use crate::record_migration::page_record_outcome_model::PageRecordOutcome;

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
        write_fixture(
            temp.path(),
            "patterns/two.md",
            &decision_page("two").replace("decision", "pattern"),
        );
        write_fixture(
            temp.path(),
            "tasks/three.md",
            "---\ntype: task\n---\n\ntask\n",
        );
        write_fixture(
            temp.path(),
            "learnings/four.md",
            "---\ntype: concept\n---\n\nlearning\n",
        );
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
        write_fixture(
            temp.path(),
            "concepts/one.md",
            "---\ntype: concept\n---\n\n## Body\n",
        );
        let first = plan_record_migration(temp.path());
        let migrated = first
            .pages
            .iter()
            .find_map(|page| match &page.outcome {
                PageRecordOutcome::WouldConvert { content, .. } => Some(content.clone()),
                _ => None,
            })
            .expect("page should be convertible");

        write_fixture(temp.path(), "concepts/one.md", &migrated);
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
        assert!(plan.notes.iter().any(|note| note.note.contains("decoded")));
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
        let migrated = first
            .pages
            .iter()
            .find_map(|page| match &page.outcome {
                PageRecordOutcome::WouldConvert { content, .. } => Some(content.clone()),
                _ => None,
            })
            .expect("page should be convertible");

        write_fixture(temp.path(), "concepts/one.md", &migrated);
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

    const STUB_PAGE: &str = "---\n{}\ntags: [alpha]\n---\n\n---\ntitle: One\nid: \"wiki:concepts:one\"\ntype: concept\n---\n\n## Body\n\nSome prose.\n";
    const STUB_TASK: &str = "---\n{}\nacceptance_criteria:\n  - {text: \"first criterion\", checked: false}\n---\n\n---\ntitle: Task\nid: \"wiki:tasks:task\"\ntype: task\nstatus: todo\n---\n\nDo the work.\n";

    fn steps_of(plan: &RecordMigrationPlan, path: &str) -> Vec<String> {
        plan.pages
            .iter()
            .find(|page| page.path == path)
            .and_then(|page| match &page.outcome {
                PageRecordOutcome::WouldConvert { steps, .. } => Some(steps.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    fn migrated_content(plan: &RecordMigrationPlan, path: &str) -> String {
        plan.pages
            .iter()
            .find(|page| page.path == path)
            .and_then(|page| match &page.outcome {
                PageRecordOutcome::WouldConvert { content, .. } => Some(content.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    #[test]
    fn a_stub_block_page_is_repaired_and_converted() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_fixture(temp.path(), "concepts/one.md", STUB_PAGE);

        let plan = plan_record_migration(temp.path());

        assert_eq!(
            steps_of(&plan, "concepts/one.md"),
            vec!["repair-frontmatter", "convert-prose"]
        );
        let content = migrated_content(&plan, "concepts/one.md");
        assert!(content.starts_with("---\ntitle: One\nid: \"wiki:concepts:one\"\ntype: concept\ntags: [alpha]\n---\n\nschema_version: 1\n"));
        assert_eq!(content.matches("title:").count(), 1);
        assert!(plan.parse_failures.is_empty());
    }

    #[test]
    fn a_stub_block_task_keeps_its_acceptance_criteria() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_fixture(temp.path(), "tasks/task.md", STUB_TASK);

        let plan = plan_record_migration(temp.path());

        let content = migrated_content(&plan, "tasks/task.md");
        assert!(content
            .contains("acceptance_criteria:\n  - {text: \"first criterion\", checked: false}"));
        assert!(content.contains("status: todo"));
        assert!(content.contains("schema_version: 1"));
    }

    #[test]
    fn a_crlf_page_converts_without_a_parse_failure() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_fixture(
            temp.path(),
            "specs/crlf.md",
            "---\r\ntitle: Crlf\r\ntype: spec\r\n---\r\n\r\n## Goal\r\n\r\nLine one.\r\n",
        );

        let plan = plan_record_migration(temp.path());

        assert!(plan.parse_failures.is_empty());
        assert_eq!(
            steps_of(&plan, "specs/crlf.md"),
            vec!["normalize-line-endings", "convert-prose"]
        );
        assert!(!migrated_content(&plan, "specs/crlf.md").contains('\r'));
    }

    #[test]
    fn a_record_newer_than_supported_is_skipped() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_fixture(
            temp.path(),
            "concepts/future.md",
            "---\ntitle: Future\ntype: concept\n---\n\nschema_version: 9\nstate: |-\n  text\n",
        );

        let plan = plan_record_migration(temp.path());

        assert_eq!(
            plan.skipped
                .get("concept")
                .and_then(|reasons| reasons.get("unsupported-version")),
            Some(&1)
        );
    }

    #[test]
    fn migrating_twice_changes_nothing_the_second_time() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_fixture(temp.path(), "concepts/one.md", STUB_PAGE);
        write_fixture(temp.path(), "tasks/task.md", STUB_TASK);
        write_fixture(temp.path(), "decisions/two.md", &decision_page("two"));
        let first = plan_record_migration(temp.path());
        let written = crate::record_migration::apply_wiki_record_migration(temp.path(), &first)
            .expect("apply");
        assert_eq!(written, 3);
        let snapshot = std::fs::read_to_string(temp.path().join("tasks/task.md")).expect("read");

        let second = plan_record_migration(temp.path());
        let rewritten = crate::record_migration::apply_wiki_record_migration(temp.path(), &second)
            .expect("apply");

        assert_eq!(rewritten, 0);
        assert_eq!(second.already_record, 3);
        assert_eq!(
            std::fs::read_to_string(temp.path().join("tasks/task.md")).expect("read"),
            snapshot
        );
    }

    #[test]
    fn every_applied_step_is_tallied() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_fixture(temp.path(), "concepts/one.md", STUB_PAGE);
        write_fixture(temp.path(), "decisions/two.md", &decision_page("two"));

        let plan = plan_record_migration(temp.path());

        assert_eq!(plan.steps.get("convert-prose"), Some(&2));
        assert_eq!(plan.steps.get("repair-frontmatter"), Some(&1));
    }
}
