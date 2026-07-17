pub mod field_change;
pub mod task_version;
pub mod task_history;
pub mod doc_version;
pub mod doc_history;
pub mod store;

pub use field_change::*;
pub use task_version::*;
pub use task_history::*;
pub use doc_version::*;
pub use doc_history::*;
pub use store::*;

pub fn compute_field_changes(old: &serde_json::Value, new: &serde_json::Value) -> Vec<field_change::FieldChange> {
    let old_map = old.as_object();
    let new_map = new.as_object();
    let mut changes = Vec::new();

    let mut fields: Vec<&str> = Vec::new();
    if let Some(m) = old_map {
        for key in m.keys() { if !fields.contains(&key.as_str()) { fields.push(key); } }
    }
    if let Some(m) = new_map {
        for key in m.keys() { if !fields.contains(&key.as_str()) { fields.push(key); } }
    }

    for field in fields {
        let old_val = old_map.and_then(|m| m.get(field));
        let new_val = new_map.and_then(|m| m.get(field));
        if old_val != new_val {
            changes.push(field_change::FieldChange {
                field: field.to_string(),
                old_value: old_val.cloned(),
                new_value: new_val.cloned(),
            });
        }
    }
    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_version_save_and_get_roundtrip() {
        let tmp = std::env::temp_dir().join("version-test");
        let _ = std::fs::remove_dir_all(&tmp);
        let store = VersionStore::new(tmp.clone());
        let changes = vec![
            FieldChange { field: "title".into(), old_value: Some(json!("Old")), new_value: Some(json!("New")) }
        ];
        store.save_task_version("task:test", changes).expect("save");
        let history = store.get_task_history("task:test").expect("get");
        assert_eq!(history.entity_id, "task:test");
        assert_eq!(history.current_version, 1);
        assert_eq!(history.versions.len(), 1);
        assert_eq!(history.versions[0].id, "v1");
        assert_eq!(history.versions[0].changes.len(), 1);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_version_save_empty_changes_noop() {
        let tmp = std::env::temp_dir().join("version-test-empty");
        let _ = std::fs::remove_dir_all(&tmp);
        let store = VersionStore::new(tmp.clone());
        store.save_task_version("task:noop", vec![]).expect("save empty");
        let history = store.get_task_history("task:noop").expect("get");
        assert_eq!(history.current_version, 0);
        assert!(history.versions.is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_fsrs_compaction_many_versions() {
        let tmp = std::env::temp_dir().join("wm-test-compaction");
        let _ = std::fs::remove_dir_all(&tmp);
        let store = VersionStore::new(tmp.clone());

        let mut history = TaskVersionHistory {
            entity_id: "task:test".into(),
            current_version: 0,
            versions: vec![],
        };
        for i in 0..100u32 {
            let ts = chrono::Utc::now() - chrono::Duration::days(60);
            history.versions.push(TaskVersion {
                id: format!("v{}", i + 1),
                version: i + 1,
                timestamp: ts.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
                author: None,
                changes: vec![],
                compacted: false,
            });
        }
        history.current_version = 100;
        let json = serde_json::to_string_pretty(&history).unwrap();
        let path = tmp.join("versions");
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join("task-test.json"), &json).unwrap();

        store
            .save_task_version(
                "task:test",
                vec![FieldChange {
                    field: "title".into(),
                    old_value: Some(json!("Old")),
                    new_value: Some(json!("New")),
                }],
            )
            .expect("save to trigger compaction");

        let loaded = store.get_task_history("task:test").unwrap();
        assert!(
            loaded.versions.len() <= 5,
            "expected ≤5 versions after compaction, got {}",
            loaded.versions.len()
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_compute_field_changes_added() {
        let old = serde_json::json!({"title": "A"});
        let new = serde_json::json!({"title": "A", "status": "done"});
        let changes = compute_field_changes(&old, &new);
        assert_eq!(changes.len(), 1, "expected 1 change");
        assert_eq!(changes[0].field, "status");
        assert_eq!(changes[0].old_value, None);
        assert_eq!(changes[0].new_value, Some(serde_json::json!("done")));
    }

    #[test]
    fn test_compute_field_changes_no_changes() {
        let val = serde_json::json!({"title": "A", "status": "done"});
        let changes = compute_field_changes(&val, &val);
        assert!(changes.is_empty(), "expected 0 changes for identical");
    }
}
