use std::path::Path;

use wm_core::version::{FieldChange, VersionStore};
use wm_migrate::{apply_wiki_record_migration_observed, MigratedPage, RecordMigrationPlan};

const CONTENT_FIELD: &str = "content";
const TASK_PAGE_TYPE: &str = "task";
const PAGE_ID_PREFIX: &str = "wiki:";
const MARKDOWN_SUFFIX: &str = ".md";

pub fn apply_with_version_history(
    wm_dir: &Path,
    wiki_dir: &Path,
    plan: &RecordMigrationPlan,
) -> std::io::Result<usize> {
    let store = VersionStore::new(wm_dir.to_path_buf());
    apply_wiki_record_migration_observed(wiki_dir, plan, &mut |page| record_version(&store, page))
}

fn page_id_of(relative_path: &str) -> String {
    let stem = relative_path
        .strip_suffix(MARKDOWN_SUFFIX)
        .unwrap_or(relative_path);
    match stem.split_once('/') {
        Some((dir, rest)) => format!("{PAGE_ID_PREFIX}{dir}:{rest}"),
        None => format!("{PAGE_ID_PREFIX}{stem}"),
    }
}

fn record_version(store: &VersionStore, page: &MigratedPage<'_>) {
    let id = page_id_of(page.path);
    let changes = vec![FieldChange {
        field: CONTENT_FIELD.to_owned(),
        old_value: Some(serde_json::Value::String(page.before.to_owned())),
        new_value: Some(serde_json::Value::String(page.after.to_owned())),
    }];
    let saved = match page.page_type == TASK_PAGE_TYPE {
        true => store.save_task_version(&id, changes),
        false => store.save_doc_version(&id, page.path, changes),
    };
    if let Err(error) = saved {
        tracing::warn!(page = %id, error = %error, "version history not saved for migrated page");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wm_migrate::plan_record_migration;

    const DECISION: &str =
        "---\ntitle: One\nid: \"wiki:decisions:one\"\ntype: decision\n---\n\n## Context\n\nprose\n";
    const TASK: &str = "---\ntitle: Task\nid: \"wiki:tasks:task\"\ntype: task\n---\n\nDo the work.\n";

    fn project_with(pages: &[(&str, &str)]) -> tempfile::TempDir {
        let temp = tempfile::tempdir().expect("tempdir");
        for (relative, content) in pages {
            let path = temp.path().join(".wm/wiki").join(relative);
            std::fs::create_dir_all(path.parent().expect("parent")).expect("dir");
            std::fs::write(path, content).expect("write");
        }
        temp
    }

    fn run(temp: &tempfile::TempDir) -> usize {
        let wm_dir = temp.path().join(".wm");
        let wiki_dir = wm_dir.join("wiki");
        let plan = plan_record_migration(&wiki_dir);
        apply_with_version_history(&wm_dir, &wiki_dir, &plan).expect("apply")
    }

    fn store(temp: &tempfile::TempDir) -> VersionStore {
        VersionStore::new(temp.path().join(".wm"))
    }

    #[test]
    fn page_ids_follow_the_wiki_naming() {
        assert_eq!(page_id_of("decisions/one.md"), "wiki:decisions:one");
        assert_eq!(page_id_of("memory/sub/a.md"), "wiki:memory:sub/a");
    }

    #[test]
    fn a_migrated_page_gets_a_content_version_with_old_and_new_text() {
        let temp = project_with(&[("decisions/one.md", DECISION)]);

        assert_eq!(run(&temp), 1);

        let history = store(&temp)
            .get_doc_history("wiki:decisions:one")
            .expect("history");
        assert_eq!(history.current_version, 1);
        let change = &history.versions[0].changes[0];
        assert_eq!(change.field, "content");
        assert_eq!(change.old_value, Some(serde_json::json!(DECISION)));
        assert!(change
            .new_value
            .as_ref()
            .and_then(|value| value.as_str())
            .is_some_and(|text| text.contains("schema_version: 1")));
    }

    #[test]
    fn a_migrated_task_is_recorded_in_the_task_history() {
        let temp = project_with(&[("tasks/task.md", TASK)]);

        assert_eq!(run(&temp), 1);

        let history = store(&temp)
            .get_task_history("wiki:tasks:task")
            .expect("history");
        assert_eq!(history.current_version, 1);
        assert_eq!(history.versions[0].changes[0].field, "content");
    }

    #[test]
    fn running_again_adds_no_version() {
        let temp = project_with(&[("decisions/one.md", DECISION)]);
        run(&temp);

        assert_eq!(run(&temp), 0);

        let history = store(&temp)
            .get_doc_history("wiki:decisions:one")
            .expect("history");
        assert_eq!(history.current_version, 1);
    }
}
