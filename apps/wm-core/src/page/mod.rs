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
    fn test_resolve_page_path_prevents_traversal() {
        let result = crate::page::resolve_page_path("test-proj", "../../etc/passwd");
        match result {
            Ok(path) => {
                assert!(
                    path.starts_with(".wm\\wiki") || path.starts_with(".wm/wiki"),
                    "path should stay within wiki dir: {:?}",
                    path
                );
            }
            Err(_) => {}
        }

        let result2 = crate::page::resolve_page_path("test-proj", "/etc/passwd");
        match result2 {
            Ok(path) => {
                assert!(
                    path.starts_with(".wm\\wiki") || path.starts_with(".wm/wiki"),
                    "path should stay within wiki dir: {:?}",
                    path
                );
            }
            Err(_) => {}
        }
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
