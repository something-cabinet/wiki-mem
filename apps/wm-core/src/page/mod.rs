pub mod helpers;
pub mod services;

pub use helpers::*;
pub use services::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_parse_yaml_mut_set_field() {
        let yaml = "title: Test\ntype: task\n";
        let result = parse_yaml_mut(yaml, |map| {
            map.insert(
                serde_yaml::Value::String("status".into()),
                serde_yaml::Value::String("done".into()),
            );
        });
        assert!(
            result.contains("status: done"),
            "Result should contain new field: {}",
            result
        );
        assert!(
            result.contains("title: Test"),
            "Result should preserve existing field: {}",
            result
        );
    }

    #[test]
    fn test_parse_yaml_mut_empty() {
        let result = parse_yaml_mut("", |map| {
            map.insert(
                serde_yaml::Value::String("key".into()),
                serde_yaml::Value::String("value".into()),
            );
        });
        assert!(
            result.contains("key: value"),
            "Empty YAML should produce new key: {}",
            result
        );
    }

    #[test]
    fn test_set_yaml_field() {
        let yaml = "title: Test\n";
        let result = set_yaml_field(yaml, "status", "done");
        assert!(result.contains("status: done"), "Result: {}", result);
        assert!(result.contains("title: Test"), "Result: {}", result);
    }

    #[test]
    fn test_ac_set_checked() {
        let yaml = r#"acceptance_criteria:
  - {text: "works", checked: false}
  - {text: "tested", checked: true}
"#;
        let result = ac_set_checked(yaml, 1, true);
        assert!(
            result.contains("checked: true"),
            "First AC should be checked: {}",
            result
        );
        assert!(
            result.contains("works"),
            "Text should be preserved: {}",
            result
        );

        let result2 = ac_set_checked(yaml, 2, false);
        assert!(
            result2.contains("checked: false"),
            "Second AC should be unchecked: {}",
            result2
        );
    }

    #[test]
    fn test_remove_yaml_block() {
        let yaml = r#"title: Test
relates_to:
  - {type: extends, target: other}
tags: [a, b]
"#;
        let result = remove_yaml_block(yaml, "relates_to");
        assert!(
            !result.contains("relates_to"),
            "Block should be removed: {}",
            result
        );
        assert!(
            result.contains("title: Test"),
            "Other fields preserved: {}",
            result
        );
        assert!(
            result.contains("tags:"),
            "Other fields preserved: {}",
            result
        );
    }

    #[test]
    fn test_title_with_colon_survives_frontmatter_roundtrip() {
        let yaml = "title: 'WM-001: Arbitrary deletion'\ntype: task\nstatus: todo\nid: wiki:tasks:wm001\n";
        let fm = crate::parser::extract_frontmatter(&format!("---\n{}---\n\nBody.\n", yaml))
            .0
            .expect("frontmatter with quoted colon title must parse");
        let out = crate::parser::frontmatter_to_yaml(&fm);
        let reparsed = crate::parser::extract_frontmatter(&format!("---\n{}---\n\nBody.\n", out))
            .0
            .expect("re-serialized frontmatter must parse again");
        assert_eq!(
            reparsed.title.as_deref(),
            Some("WM-001: Arbitrary deletion"),
            "colon-bearing title must survive round-trip, got: {}",
            out
        );
        assert_eq!(reparsed.page_type.as_deref(), Some("task"), "type lost: {}", out);
    }

    #[test]
    fn test_yaml_scalar_quotes_colon_values() {
        let quoted = crate::page::helpers::yaml_helper::yaml_scalar("WM-001: thing");
        assert!(
            quoted.starts_with('\'') || quoted.starts_with('"'),
            "value containing ':' must be quoted, got: {}",
            quoted
        );
        assert_eq!(crate::page::helpers::yaml_helper::yaml_scalar("Plain"), "Plain");
    }

    #[test]
    fn test_set_yaml_field_preserves_scientific_notation_id() {
        let yaml = "title: Test\ntype: task\nid: 652e07\nstatus: todo\n";
        let result = set_yaml_field(yaml, "status", "done");
        assert!(
            result.contains("id: 652e07"),
            "unquoted scientific-notation id must be preserved byte-for-byte, got: {}",
            result
        );
        assert!(result.contains("status: done"), "status updated, got: {}", result);
        assert!(result.contains("title: Test"), "title preserved, got: {}", result);
    }

    #[test]
    fn test_set_yaml_field_quotes_id_value() {
        let yaml = "title: T\n";
        let result = set_yaml_field(yaml, "id", "652e07");
        assert!(
            result.contains("id: '652e07'") || result.contains("id: \"652e07\""),
            "id value must be quoted on write to survive YAML round-trips, got: {}",
            result
        );
    }

    #[test]
    fn test_remove_yaml_block_preserves_other_fields_without_roundtrip() {
        let yaml = "title: T\nid: 652e07\nrelates_to:\n  - {type: extends, target: x}\nstatus: todo\n";
        let result = remove_yaml_block(yaml, "relates_to");
        assert!(
            result.contains("id: 652e07"),
            "id must survive remove_yaml_block without a serde_yaml round-trip, got: {}",
            result
        );
        assert!(!result.contains("relates_to"), "block removed, got: {}", result);
        assert!(result.contains("status: todo"), "status preserved, got: {}", result);
    }

    #[test]
    fn test_no_write_path_emits_empty_block() {
        let result = parse_yaml_mut("", |_map| {});
        assert!(
            !result.contains("{}"),
            "no-op parse_yaml_mut on empty input must not emit '{{}}', got: {:?}",
            result
        );
        let result2 = remove_yaml_block("", "tags");
        assert!(
            !result2.contains("{}"),
            "remove_yaml_block on empty input must not emit '{{}}', got: {:?}",
            result2
        );
    }

    #[test]
    fn test_resolve_page_path_prevents_traversal() {
        let result = crate::page::resolve_page_path("test-proj", "../../etc/passwd");
        assert!(
            result.is_err(),
            "leading-parent traversal must be rejected, got: {:?}",
            result
        );

        let result2 = crate::page::resolve_page_path("test-proj", "/etc/passwd");
        assert!(
            result2.is_err(),
            "absolute path outside the wiki root must be rejected, got: {:?}",
            result2
        );

        let ok = crate::page::resolve_page_path("test-proj", "tasks/valid-task")
            .expect("valid page path must resolve");
        assert!(
            ok.starts_with(".wm\\wiki") || ok.starts_with(".wm/wiki"),
            "valid path must stay within wiki dir, got: {:?}",
            ok
        );
    }

    #[test]
    fn test_set_yaml_value_field_roundtrips_all_types() {
        use crate::page::helpers::yaml_helper::set_yaml_value_field;

        let mut yaml = "title: Test\nstatus: draft\n".to_string();
        yaml = set_yaml_value_field(&yaml, "knowns_id", &serde_json::json!("legacy-007"));
        yaml = set_yaml_value_field(&yaml, "confidence", &serde_json::json!("high"));
        yaml = set_yaml_value_field(&yaml, "aliases", &serde_json::json!(["alpha", "beta"]));
        yaml = set_yaml_value_field(&yaml, "single", &serde_json::json!(["only"]));
        yaml = set_yaml_value_field(&yaml, "nested", &serde_json::json!({"depth": 2, "ok": true}));

        let parsed: serde_yaml::Value =
            serde_yaml::from_str(&yaml).expect("updated frontmatter must be valid YAML");
        let map = parsed.as_mapping().expect("frontmatter must be a mapping");
        let get = |k: &str| map.get(serde_yaml::Value::String(k.to_string()));
        assert_eq!(get("knowns_id").and_then(|v| v.as_str()), Some("legacy-007"));
        assert_eq!(get("confidence").and_then(|v| v.as_str()), Some("high"));
        assert_eq!(
            get("aliases").and_then(|v| v.as_sequence()).map(|s| s.len()),
            Some(2)
        );
        assert_eq!(
            get("single").and_then(|v| v.as_sequence()).map(|s| s.len()),
            Some(1),
            "single-element list must round-trip: {}",
            yaml
        );
        let nested = get("nested").and_then(|v| v.as_mapping());
        assert_eq!(
            nested
                .and_then(|m| m.get(serde_yaml::Value::String("depth".into())))
                .and_then(|v| v.as_i64()),
            Some(2)
        );

        let yaml = set_yaml_value_field(&yaml, "status", &serde_json::json!("done"));
        assert!(yaml.contains("status: done"), "replace existing: {yaml}");
    }

    #[test]
    fn test_update_page_extra_frontmatter_and_type_persist() {
        let tmp = std::env::temp_dir().join("wm-test-extra-fm-unit");
        let _ = std::fs::remove_dir_all(&tmp);
        let wiki_dir = tmp.join(".wm").join("wiki").join("concepts");
        std::fs::create_dir_all(&wiki_dir).unwrap();
        let page_path = wiki_dir.join("unit-extra.md");
        std::fs::write(
            &page_path,
            "---\ntitle: Unit Extra\ntype: concept\nid: \"wiki:concepts:unit-extra\"\nstatus: draft\n---\n\nBody.\n",
        )
        .unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        let (engine, _rx) =
            crate::engine::EngineState::new(crate::config::ProjectConfig::default(), tmp.clone());
        let engine = Arc::new(engine);

        let params = crate::page::PageUpdateParams {
            r#type: Some("pattern".into()),
            extra_frontmatter: Some({
                let mut m = serde_json::Map::new();
                m.insert("knowns_id".into(), serde_json::json!("legacy-009"));
                m.insert("confidence".into(), serde_json::json!("high"));
                m.insert("aliases".into(), serde_json::json!(["a", "b"]));
                m
            }),
            ..Default::default()
        };
        crate::page::update_page(&engine, "wiki:concepts:unit-extra", &params)
            .expect("update with extra frontmatter");

        let content = std::fs::read_to_string(&page_path).unwrap();
        assert!(
            content.contains("type: pattern"),
            "type param must write frontmatter type: {content}"
        );
        assert!(
            content.contains("knowns_id: legacy-009"),
            "knowns_id must persist: {content}"
        );
        assert!(
            content.contains("confidence: high"),
            "confidence must persist: {content}"
        );
        assert!(content.contains("aliases:"), "aliases must persist: {content}");
        assert!(content.contains("- a"), "aliases item must persist: {content}");
        assert!(content.contains("Body."), "body must be preserved: {content}");

        let (fm, _) = crate::parser::extract_frontmatter(&content);
        let fm = fm.expect("frontmatter must parse after update");
        assert_eq!(fm.page_type.as_deref(), Some("pattern"), "type parse-back");
        assert_eq!(fm.title.as_deref(), Some("Unit Extra"), "title preserved");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_migrate_no_memory_dir() {
        let tmp = std::env::temp_dir().join("wm-test-no-mem");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join(".wm").join("wiki")).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        let (engine, _rx) =
            crate::engine::EngineState::new(crate::config::ProjectConfig::default(), tmp.clone());
        let engine = Arc::new(engine);

        let result = crate::page::migrate_old_memory_json(&engine);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_migrate_invalid_json() {
        let tmp = std::env::temp_dir().join("wm-test-bad-json");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join(".wm").join("memory")).unwrap();
        std::fs::write(
            tmp.join(".wm").join("memory").join("bad.json"),
            "not valid json",
        )
        .unwrap();
        std::fs::create_dir_all(tmp.join(".wm").join("wiki")).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        let (engine, _rx) =
            crate::engine::EngineState::new(crate::config::ProjectConfig::default(), tmp.clone());
        let engine = Arc::new(engine);

        let result = crate::page::migrate_old_memory_json(&engine);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            0,
            "should migrate 0 files when all are invalid"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
