use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::engine::EngineState;
use crate::error::{ToolError, ToolResult};
use crate::page;

// ─── Types ───────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FieldChange {
    pub field: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskVersion {
    pub id: String,
    pub version: u32,
    pub timestamp: String,
    pub author: Option<String>,
    pub changes: Vec<FieldChange>,
    pub compacted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskVersionHistory {
    pub entity_id: String,
    pub current_version: u32,
    pub versions: Vec<TaskVersion>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocVersion {
    pub id: String,
    pub version: u32,
    pub timestamp: String,
    pub author: Option<String>,
    pub changes: Vec<FieldChange>,
    pub path: String,
    pub compacted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocVersionHistory {
    pub entity_id: String,
    pub current_version: u32,
    pub versions: Vec<DocVersion>,
}

// ─── VersionStore ────────────────────────────────────────

pub struct VersionStore {
    root: PathBuf,
}

impl VersionStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn task_path(&self, task_id: &str) -> PathBuf {
        let safe = task_id.replace(':', "-");
        self.root.join("versions").join(format!("task-{safe}.json"))
    }

    fn doc_path(&self, doc_id: &str) -> PathBuf {
        let safe = doc_id.replace([':', '/', '\\'], "-");
        self.root.join("versions").join(format!("doc-{safe}.json"))
    }

    fn now() -> String {
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
    }

    // FSRS recency score: R(t) = 1 / (1 + t/days) with days from config stability
    #[allow(dead_code)]
    fn fsrs_score(&self, timestamp: &str, stability_days: f64) -> f64 {
        let Ok(ts) = chrono::DateTime::parse_from_rfc3339(timestamp) else { return 1.0 };
        let ts_utc = ts.with_timezone(&chrono::Utc);
        let age_days = (chrono::Utc::now() - ts_utc).num_hours() as f64 / 24.0;
        1.0 / (1.0 + age_days / stability_days.max(1.0))
    }

    // ─── Task versions ────────────────────────────────

    pub fn save_task_version(&self, task_id: &str, changes: Vec<FieldChange>) -> ToolResult<()> {
        self.save_task_version_inner(task_id, changes, None)
    }

    fn save_task_version_inner(&self, task_id: &str, changes: Vec<FieldChange>, author: Option<&str>) -> ToolResult<()> {
        if changes.is_empty() { return Ok(()); }
        let path = self.task_path(task_id);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut history: TaskVersionHistory = if path.exists() {
            let data = std::fs::read_to_string(&path)?;
            serde_json::from_str(&data).unwrap_or(TaskVersionHistory {
                entity_id: task_id.to_string(),
                current_version: 0,
                versions: vec![],
            })
        } else {
            TaskVersionHistory {
                entity_id: task_id.to_string(),
                current_version: 0,
                versions: vec![],
            }
        };

        history.current_version += 1;
        let version = TaskVersion {
            id: format!("v{}", history.current_version),
            version: history.current_version,
            timestamp: Self::now(),
            author: author.map(String::from),
            changes,
            compacted: false,
        };
        history.versions.push(version);

        // Apply FSRS compaction
        self.compact_task_history(&mut history);

        std::fs::write(&path, serde_json::to_string_pretty(&history)?)?;
        Ok(())
    }

    pub fn get_task_history(&self, task_id: &str) -> ToolResult<TaskVersionHistory> {
        let path = self.task_path(task_id);
        if !path.exists() {
            return Ok(TaskVersionHistory {
                entity_id: task_id.to_string(),
                current_version: 0,
                versions: vec![],
            });
        }
        let data = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&data)?)
    }

    // ─── Doc versions ─────────────────────────────────

    pub fn save_doc_version(&self, doc_id: &str, doc_path: &str, changes: Vec<FieldChange>) -> ToolResult<()> {
        self.save_doc_version_inner(doc_id, doc_path, changes, None)
    }

    fn save_doc_version_inner(&self, doc_id: &str, doc_path: &str, changes: Vec<FieldChange>, author: Option<&str>) -> ToolResult<()> {
        if changes.is_empty() { return Ok(()); }
        let path = self.doc_path(doc_id);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut history: DocVersionHistory = if path.exists() {
            let data = std::fs::read_to_string(&path)?;
            serde_json::from_str(&data).unwrap_or(DocVersionHistory {
                entity_id: doc_id.to_string(),
                current_version: 0,
                versions: vec![],
            })
        } else {
            DocVersionHistory {
                entity_id: doc_id.to_string(),
                current_version: 0,
                versions: vec![],
            }
        };

        history.current_version += 1;
        let version = DocVersion {
            id: format!("v{}", history.current_version),
            version: history.current_version,
            timestamp: Self::now(),
            author: author.map(String::from),
            changes,
            path: doc_path.to_string(),
            compacted: false,
        };
        history.versions.push(version);

        self.compact_doc_history(&mut history);

        std::fs::write(&path, serde_json::to_string_pretty(&history)?)?;
        Ok(())
    }

    pub fn get_doc_history(&self, doc_id: &str) -> ToolResult<DocVersionHistory> {
        let path = self.doc_path(doc_id);
        if !path.exists() {
            return Ok(DocVersionHistory {
                entity_id: doc_id.to_string(),
                current_version: 0,
                versions: vec![],
            });
        }
        let data = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&data)?)
    }

    // ─── Compaction ───────────────────────────────────

    fn compact_task_history(&self, history: &mut TaskVersionHistory) {
        let stability = 7.0; // days — should come from config eventually
        // Keep last 3 versions always (most recent changes)
        let keep = 3usize;
        if history.versions.len() <= keep { return; }

        let mut compacted: Vec<TaskVersion> = Vec::new();
        // Keep the last `keep` versions intact
        let split_point = history.versions.len().saturating_sub(keep);
        let recent = history.versions.split_off(split_point);

        // Compact old versions with low FSRS score
        #[allow(unused_variables)]
        let old_count = history.versions.len();
        let compacted_count = history.versions.iter()
            .filter(|v| self.fsrs_score(&v.timestamp, stability) < 0.15)
            .count();

        if compacted_count > 0 && compacted_count == old_count && old_count > 0 {
            // All old versions are ancient — collapse to one gap entry
            compacted.push(TaskVersion {
                id: "v0-compacted".to_string(),
                version: 0,
                timestamp: history.versions.last().map(|v| v.timestamp.clone()).unwrap_or_default(),
                author: None,
                changes: vec![],
                compacted: true,
            });
        } else if compacted_count > 0 {
            // Keep non-compacted old versions, compact the rest
            for v in history.versions.drain(..) {
                if self.fsrs_score(&v.timestamp, stability) < 0.15 {
                    if !v.compacted {
                        // skip — drop the entry entirely (too old)
                    } else {
                        compacted.push(v);
                    }
                } else {
                    compacted.push(v);
                }
            }
        }

        compacted.extend(recent);
        history.versions = compacted;
    }

    #[allow(dead_code)]
    fn compact_doc_history(&self, _history: &mut DocVersionHistory) {
        // Same logic as task but for docs — simplified for now
    }

    // ─── Rollback ─────────────────────────────────────────

    /// Rollback a task to a previous version by applying inverse changes
    /// from current version down to `target_version + 1`, then record the
    /// rollback as a new version entry.
    pub fn rollback_task(&self, task_id: &str, target_version: u32, engine: &Arc<EngineState>) -> ToolResult<()> {
        let history = self.get_task_history(task_id)?;

        if history.versions.is_empty() {
            return Err(ToolError::not_found("version", &format!("task {}", task_id)));
        }

        // Validate target version exists
        let _target = history
            .versions
            .iter()
            .find(|v| v.version == target_version)
            .ok_or_else(|| {
                ToolError::not_found(
                    "version",
                    &format!("version {} for task {}", target_version, task_id),
                )
            })?;

        if target_version >= history.current_version {
            return Err(ToolError::invalid_params(format!(
                "Target version {} must be less than current version {}",
                target_version, history.current_version
            )));
        }

        // Accumulate inverse changes from current down to target+1
        let mut params = page::PageUpdateParams::default();
        let mut rollback_changes: Vec<FieldChange> = Vec::new();

        for version in history.versions.iter().rev() {
            if version.version <= target_version {
                break;
            }
            for change in &version.changes {
                // Track the net inverse change for the version entry
                rollback_changes.push(FieldChange {
                    field: change.field.clone(),
                    old_value: change.new_value.clone(),
                    new_value: change.old_value.clone(),
                });

                // Build PageUpdateParams — later (older) entries overwrite
                match change.field.as_str() {
                    "title" => {
                        params.title = change
                            .old_value
                            .as_ref()
                            .and_then(|v| v.as_str().map(String::from));
                    }
                    "status" => {
                        params.status = change
                            .old_value
                            .as_ref()
                            .and_then(|v| v.as_str().map(String::from));
                    }
                    "priority" => {
                        params.priority = change
                            .old_value
                            .as_ref()
                            .and_then(|v| v.as_str().map(String::from));
                    }
                    "assignee" => {
                        params.assignee = change
                            .old_value
                            .as_ref()
                            .and_then(|v| v.as_str().map(String::from));
                    }
                    "content" | "description" => {
                        params.content = change
                            .old_value
                            .as_ref()
                            .and_then(|v| v.as_str().map(String::from));
                    }
                    "implementation_plan" => {
                        params.implementation_plan = change
                            .old_value
                            .as_ref()
                            .and_then(|v| v.as_str().map(String::from));
                    }
                    "implementation_notes" => {
                        params.implementation_notes = change
                            .old_value
                            .as_ref()
                            .and_then(|v| v.as_str().map(String::from));
                    }
                    "tags" => {
                        params.tags = change
                            .old_value
                            .as_ref()
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            });
                    }
                    _ => {
                        // Unknown fields — page::update_page won't touch them,
                        // they remain as-is in the YAML frontmatter.
                    }
                }
            }
        }

        // Apply restored state via page::update_page
        page::update_page(engine, task_id, &params)?;

        // Record rollback as a new version (author = "rollback")
        self.save_task_version_inner(task_id, rollback_changes, Some("rollback"))?;

        Ok(())
    }

    /// Rollback a doc (non-task wiki page) to a previous version.
    /// Uses the same inverse-change accumulation as rollback_task,
    /// applying changes via page::update_page since docs are wiki pages too.
    pub fn rollback_doc(&self, doc_id: &str, target_version: u32, engine: &Arc<EngineState>) -> ToolResult<()> {
        let history = self.get_doc_history(doc_id)?;

        if history.versions.is_empty() {
            return Err(ToolError::not_found("version", &format!("doc {}", doc_id)));
        }

        // Validate target version exists
        let _target = history
            .versions
            .iter()
            .find(|v| v.version == target_version)
            .ok_or_else(|| {
                ToolError::not_found(
                    "version",
                    &format!("version {} for doc {}", target_version, doc_id),
                )
            })?;

        if target_version >= history.current_version {
            return Err(ToolError::invalid_params(format!(
                "Target version {} must be less than current version {}",
                target_version, history.current_version
            )));
        }

        // Accumulate inverse changes from current down to target+1
        let mut params = page::PageUpdateParams::default();
        let mut rollback_changes: Vec<FieldChange> = Vec::new();

        for version in history.versions.iter().rev() {
            if version.version <= target_version {
                break;
            }
            for change in &version.changes {
                rollback_changes.push(FieldChange {
                    field: change.field.clone(),
                    old_value: change.new_value.clone(),
                    new_value: change.old_value.clone(),
                });

                match change.field.as_str() {
                    "title" => {
                        params.title = change
                            .old_value
                            .as_ref()
                            .and_then(|v| v.as_str().map(String::from));
                    }
                    "status" => {
                        params.status = change
                            .old_value
                            .as_ref()
                            .and_then(|v| v.as_str().map(String::from));
                    }
                    "priority" => {
                        params.priority = change
                            .old_value
                            .as_ref()
                            .and_then(|v| v.as_str().map(String::from));
                    }
                    "assignee" => {
                        params.assignee = change
                            .old_value
                            .as_ref()
                            .and_then(|v| v.as_str().map(String::from));
                    }
                    "content" | "description" => {
                        params.content = change
                            .old_value
                            .as_ref()
                            .and_then(|v| v.as_str().map(String::from));
                    }
                    "tags" => {
                        params.tags = change
                            .old_value
                            .as_ref()
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            });
                    }
                    _ => {}
                }
            }
        }

        // Apply restored state via page::update_page
        page::update_page(engine, doc_id, &params)?;

        // Record rollback as a new version
        // Resolve current doc path from the graph for the version entry
        let doc_path = {
            let snapshot = engine.graph.load();
            let index = &snapshot.1;
            index.get(doc_id).map(|idx| {
                let meta = &snapshot.0[*idx];
                let root = engine
                    .project_root
                    .read()
                    .map(|r| r.clone())
                    .unwrap_or_default();
                meta.path
                    .strip_prefix(&root)
                    .unwrap_or(&meta.path)
                    .to_string_lossy()
                    .to_string()
            })
        };
        self.save_doc_version_inner(
            doc_id,
            doc_path.as_deref().unwrap_or(""),
            rollback_changes,
            Some("rollback"),
        )?;

        Ok(())
    }
}

// ─── Helper: compute changes between two structs ───────────

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

        // Create versions with old timestamps (60 days ago — all FSRS < 0.15)
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

        // Trigger compaction by saving a new version
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

        // Load and check compaction reduced size
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

// ─── Helper: compute changes between two structs ───────────

/// Compare old and new values of tracked fields, producing a list of changes.
/// `old` and `new` are maps of field_name → serde_json::Value.
#[allow(dead_code)]
pub fn compute_field_changes(old: &serde_json::Value, new: &serde_json::Value) -> Vec<FieldChange> {
    let old_map = old.as_object();
    let new_map = new.as_object();
    let mut changes = Vec::new();

    // Collect all field names
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
            changes.push(FieldChange {
                field: field.to_string(),
                old_value: old_val.cloned(),
                new_value: new_val.cloned(),
            });
        }
    }
    changes
}
