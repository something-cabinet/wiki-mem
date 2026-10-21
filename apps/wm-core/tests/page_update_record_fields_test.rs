use std::sync::Arc;

use wm_core::config::ProjectConfig;
use wm_core::engine::{AcceptanceCriterion, EngineState};
use wm_core::page::{update_page, PageUpdateParams};

const PAGE_ID: &str = "wiki:tasks:hostile-fields";

fn seed_page(root: &std::path::Path) -> std::path::PathBuf {
    let wiki_dir = root.join(".wm").join("wiki").join("tasks");
    std::fs::create_dir_all(&wiki_dir).expect("create wiki dir");
    let page_path = wiki_dir.join("hostile-fields.md");
    std::fs::write(
        &page_path,
        "---\ntitle: Hostile Fields\ntype: task\nid: \"wiki:tasks:hostile-fields\"\nstatus: todo\n---\n\nBody.\n",
    )
    .expect("seed page");
    page_path
}

#[tokio::test(flavor = "multi_thread")]
async fn hostile_tags_and_acceptance_criteria_round_trip() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().to_path_buf();
    let page_path = seed_page(&root);
    let (engine, _rx) = EngineState::new(ProjectConfig::default(), root);
    let engine = Arc::new(engine);
    let tags = vec![
        "[leading".to_string(),
        "colon: value".to_string(),
        "quote\"inside".to_string(),
    ];
    let params = PageUpdateParams {
        tags: Some(tags.clone()),
        acceptance_criteria: Some(vec![
            AcceptanceCriterion {
                text: "[bracket]: \"quoted\" value".into(),
                checked: false,
            },
            AcceptanceCriterion {
                text: "plain: text".into(),
                checked: true,
            },
        ]),
        ..Default::default()
    };

    update_page(&engine, PAGE_ID, &params).expect("update with hostile record fields");

    let content = std::fs::read_to_string(&page_path).expect("read page");
    let (raw_fm, _) = wm_engine::parser::extract_raw_frontmatter(&content);
    let parsed: serde_yaml::Value = serde_yaml::from_str(&raw_fm.expect("frontmatter present"))
        .expect("hostile values must stay valid YAML");
    let map = parsed.as_mapping().expect("frontmatter mapping");
    let get = |key: &str| map.get(serde_yaml::Value::String(key.to_string()));

    let stored_tags: Vec<&str> = get("tags")
        .and_then(|value| value.as_sequence())
        .expect("tags sequence")
        .iter()
        .filter_map(|value| value.as_str())
        .collect();
    assert_eq!(
        stored_tags, tags,
        "tags must round-trip verbatim:\n{content}"
    );

    let stored_ac = get("acceptance_criteria")
        .and_then(|value| value.as_sequence())
        .expect("acceptance_criteria sequence");
    assert_eq!(stored_ac.len(), 2, "both criteria must persist:\n{content}");
    assert_eq!(
        stored_ac[0]
            .as_mapping()
            .and_then(|criterion| criterion.get(serde_yaml::Value::String("text".into())))
            .and_then(|value| value.as_str()),
        Some("[bracket]: \"quoted\" value")
    );
    assert_eq!(
        stored_ac[1]
            .as_mapping()
            .and_then(|criterion| criterion.get(serde_yaml::Value::String("checked".into())))
            .and_then(|value| value.as_bool()),
        Some(true)
    );
}
