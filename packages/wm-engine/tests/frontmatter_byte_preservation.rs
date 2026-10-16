use std::path::{Path, PathBuf};

use serde_yaml::Value;
use wm_engine::parser::{extract_frontmatter, extract_raw_frontmatter, frontmatter_to_yaml};

fn tasks_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(".wm/wiki/tasks")
}

fn timer_pages() -> Vec<(String, String)> {
    let dir = tasks_dir();
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("live tasks directory")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .filter_map(|path| {
            let content = std::fs::read_to_string(&path).ok()?;
            let raw = extract_raw_frontmatter(&content).0?;
            let carries_timer = raw.lines().any(|line| line.starts_with("time_started:"));
            let name = path.file_name()?.to_string_lossy().to_string();
            carries_timer.then_some((name, content))
        })
        .collect()
}

fn source_scalar(raw: &str, key: &str) -> Option<String> {
    let value: Value = serde_yaml::from_str(raw).ok()?;
    value.get(key)?.as_str().map(str::to_string)
}

fn source_relates(raw: &str) -> Vec<(String, String)> {
    let Ok(value) = serde_yaml::from_str::<Value>(raw) else {
        return Vec::new();
    };
    let Some(sequence) = value.get("relates_to").and_then(Value::as_sequence) else {
        return Vec::new();
    };
    sequence
        .iter()
        .filter_map(|entry| {
            let edge = entry.get("type")?.as_str()?.to_string();
            let target = entry.get("target")?.as_str()?.to_string();
            Some((edge, target))
        })
        .collect()
}

fn emit_from_fragment(fragment: &str) -> String {
    let content = format!("---\n{}---\n\nbody\n", fragment);
    let meta = extract_frontmatter(&content)
        .0
        .expect("synthetic fragment must parse");
    frontmatter_to_yaml(&meta)
}

#[test]
fn live_timer_pages_preserve_source_scalars_and_relations() {
    let pages = timer_pages();
    assert!(
        !pages.is_empty(),
        "no live task pages carrying time_started"
    );
    for (name, content) in &pages {
        let raw = extract_raw_frontmatter(content)
            .0
            .expect("live page has frontmatter");
        let meta = extract_frontmatter(content)
            .0
            .expect("live page frontmatter parses");
        assert_eq!(meta.raw_type, source_scalar(&raw, "type"), "{} type", name);
        assert_eq!(
            meta.raw_status,
            source_scalar(&raw, "status"),
            "{} status",
            name
        );
        assert_eq!(
            meta.raw_priority,
            source_scalar(&raw, "priority"),
            "{} priority",
            name
        );
        assert_eq!(
            meta.raw_confidence,
            source_scalar(&raw, "confidence"),
            "{} confidence",
            name
        );
        assert_eq!(
            meta.raw_relates_to,
            source_relates(&raw),
            "{} relates_to",
            name
        );
    }
}

#[test]
fn live_timer_pages_absent_keys_stay_absent_and_raw_values_reemit() {
    let pages = timer_pages();
    assert!(
        !pages.is_empty(),
        "no live task pages carrying time_started"
    );
    for (name, content) in &pages {
        let meta = extract_frontmatter(content)
            .0
            .expect("live page frontmatter parses");
        let emitted = frontmatter_to_yaml(&meta);
        for (key, raw_value) in [
            ("type", &meta.raw_type),
            ("status", &meta.raw_status),
            ("priority", &meta.raw_priority),
            ("confidence", &meta.raw_confidence),
        ] {
            let emitted_line = format!("{}: {}", key, raw_value.clone().unwrap_or_default());
            let emitted_has_key = emitted
                .lines()
                .any(|line| line.starts_with(&format!("{}: ", key)));
            assert_eq!(
                emitted_has_key,
                raw_value.is_some(),
                "{}: presence of {} key in re-emit must match source",
                name,
                key
            );
            if raw_value.is_some() {
                assert!(
                    emitted.lines().any(|line| line == emitted_line),
                    "{}: re-emit must contain raw line {:?}",
                    name,
                    emitted_line
                );
            }
        }
    }
}

#[test]
fn serializer_pins_synthetic_shapes() {
    assert_eq!(
        emit_from_fragment("title: T\nid: wiki:x\n"),
        "title: T\nid: \"wiki:x\"\n"
    );
    assert_eq!(
        emit_from_fragment("title: T\ntype: Guide\nstatus: WIP\npriority: critical\n"),
        "title: T\ntype: Guide\nstatus: WIP\npriority: critical\n"
    );
    assert_eq!(
        emit_from_fragment(
            "title: T\nrelates_to:\n  - {type: relates-to, target: a}\n  - {type: example-of, target: b}\n"
        ),
        "title: T\nrelates_to:\n  - {type: relates-to, target: a}\n  - {type: example-of, target: b}\n"
    );
    assert_eq!(
        emit_from_fragment("title: T\nrationale: because\n"),
        "title: T\nrationale: \"because\"\n"
    );
    assert_eq!(
        emit_from_fragment("title: T\ncategory: testing\nrationale: because\n"),
        "title: T\ncategory: Testing\nrationale: \"because\"\n"
    );
}
