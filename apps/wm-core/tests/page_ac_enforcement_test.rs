use std::path::{Path, PathBuf};
use std::sync::Arc;

use wm_core::config::ProjectConfig;
use wm_core::engine::EngineState;
use wm_core::page::{create_page, update_page, PageUpdateParams};

const TASK_WITH_AC: &str = "---\ntitle: Seed Task\ntype: task\nid: \"wiki:tasks:seed-task\"\nstatus: todo\nacceptance_criteria:\n  - text: \"criterion\"\n---\n\nBody.\n";
const SPEC_WITH_AC: &str = "---\ntitle: Seed Spec\ntype: spec\nid: \"wiki:specs:seed-spec\"\nstatus: draft\nacceptance_criteria:\n  - text: \"criterion\"\n---\n\nBody.\n";

fn engine(root: &Path) -> Arc<EngineState> {
    let (engine, _rx) = EngineState::new(ProjectConfig::default(), root.to_path_buf());
    Arc::new(engine)
}

fn seed(root: &Path, dir: &str, stem: &str, content: &str) -> PathBuf {
    let wiki_dir = root.join(".wm").join("wiki").join(dir);
    std::fs::create_dir_all(&wiki_dir).expect("create wiki dir");
    let path = wiki_dir.join(format!("{stem}.md"));
    std::fs::write(&path, content).expect("seed page");
    path
}

#[tokio::test(flavor = "multi_thread")]
async fn create_task_without_acceptance_criteria_is_rejected() {
    let dir = tempfile::tempdir().expect("temp dir");
    let engine = engine(dir.path());
    let err = create_page(
        &engine,
        "tasks/no-ac-task",
        "title: No AC Task\ntype: task\nstatus: todo\n",
        "Body.",
    )
    .expect_err("task without acceptance criteria must be rejected");
    assert_eq!(err.code, "INVALID_PARAMS");
    assert!(err.message.contains("task page"), "got: {}", err.message);
    assert!(
        err.message.contains("acceptance_criteria"),
        "got: {}",
        err.message
    );
    assert!(
        !dir.path().join(".wm/wiki/tasks/no-ac-task.md").exists(),
        "rejected create must not write the page"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn create_task_with_acceptance_criteria_succeeds() {
    let dir = tempfile::tempdir().expect("temp dir");
    let engine = engine(dir.path());
    create_page(
        &engine,
        "tasks/with-ac-task",
        "title: With AC Task\ntype: task\nstatus: todo\nacceptance_criteria:\n  - text: \"criterion\"\n",
        "Body.",
    )
    .expect("task with acceptance criteria must be created");
    assert!(dir.path().join(".wm/wiki/tasks/with-ac-task.md").exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn create_spec_without_acceptance_criteria_is_rejected() {
    let dir = tempfile::tempdir().expect("temp dir");
    let engine = engine(dir.path());
    let err = create_page(
        &engine,
        "specs/no-ac-spec",
        "title: No AC Spec\ntype: spec\nstatus: draft\n",
        "Body.",
    )
    .expect_err("spec without acceptance criteria must be rejected");
    assert_eq!(err.code, "INVALID_PARAMS");
    assert!(err.message.contains("spec page"), "got: {}", err.message);
    assert!(
        err.message.contains("acceptance_criteria"),
        "got: {}",
        err.message
    );
    assert!(!dir.path().join(".wm/wiki/specs/no-ac-spec.md").exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn create_spec_with_acceptance_criteria_succeeds() {
    let dir = tempfile::tempdir().expect("temp dir");
    let engine = engine(dir.path());
    create_page(
        &engine,
        "specs/with-ac-spec",
        "title: With AC Spec\ntype: spec\nstatus: draft\nacceptance_criteria:\n  - text: \"criterion\"\n",
        "Body.",
    )
    .expect("spec with acceptance criteria must be created");
    assert!(dir.path().join(".wm/wiki/specs/with-ac-spec.md").exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn create_concept_without_acceptance_criteria_succeeds() {
    let dir = tempfile::tempdir().expect("temp dir");
    let engine = engine(dir.path());
    create_page(
        &engine,
        "concepts/plain-concept",
        "title: Plain Concept\ntype: concept\nstatus: draft\n",
        "Body.",
    )
    .expect("non-enforced page type must stay optional");
}

#[tokio::test(flavor = "multi_thread")]
async fn update_emptying_task_acceptance_criteria_is_rejected() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = seed(dir.path(), "tasks", "seed-task", TASK_WITH_AC);
    let before = std::fs::read_to_string(&path).expect("read page");
    let engine = engine(dir.path());
    let params = PageUpdateParams {
        acceptance_criteria: Some(Vec::new()),
        ..Default::default()
    };
    let err = update_page(&engine, "wiki:tasks:seed-task", &params)
        .expect_err("emptying task acceptance criteria must be rejected");
    assert_eq!(err.code, "INVALID_PARAMS");
    assert!(err.message.contains("task page"), "got: {}", err.message);
    assert_eq!(
        std::fs::read_to_string(&path).expect("read page"),
        before,
        "rejected update must leave the page untouched"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn update_emptying_spec_acceptance_criteria_is_rejected() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = seed(dir.path(), "specs", "seed-spec", SPEC_WITH_AC);
    let before = std::fs::read_to_string(&path).expect("read page");
    let engine = engine(dir.path());
    let params = PageUpdateParams {
        acceptance_criteria: Some(Vec::new()),
        ..Default::default()
    };
    let err = update_page(&engine, "wiki:specs:seed-spec", &params)
        .expect_err("emptying spec acceptance criteria must be rejected");
    assert_eq!(err.code, "INVALID_PARAMS");
    assert!(err.message.contains("spec page"), "got: {}", err.message);
    assert_eq!(
        std::fs::read_to_string(&path).expect("read page"),
        before,
        "rejected update must leave the page untouched"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn update_task_keeping_acceptance_criteria_succeeds() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = seed(dir.path(), "tasks", "seed-task", TASK_WITH_AC);
    let engine = engine(dir.path());
    let params = PageUpdateParams {
        title: Some("Renamed Task".into()),
        ..Default::default()
    };
    update_page(&engine, "wiki:tasks:seed-task", &params).expect("update keeps existing criteria");
    let content = std::fs::read_to_string(&path).expect("read page");
    assert!(content.contains("title: Renamed Task"), "got:\n{content}");
}
