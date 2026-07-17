use std::path::PathBuf;
use std::sync::Arc;

use crate::engine::EngineState;
use wm_error::{ToolError, ToolResult};
use crate::page;

use super::field_change_model::FieldChange;
use super::task_version_model::TaskVersion;
use super::task_version_history_model::TaskVersionHistory;
use super::doc_version_model::DocVersion;
use super::doc_version_history_model::DocVersionHistory;

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

    #[allow(dead_code)]
    fn fsrs_score(&self, timestamp: &str, stability_days: f64) -> f64 {
        let Ok(ts) = chrono::DateTime::parse_from_rfc3339(timestamp) else { return 1.0 };
        let ts_utc = ts.with_timezone(&chrono::Utc);
        let age_days = (chrono::Utc::now() - ts_utc).num_hours() as f64 / 24.0;
        1.0 / (1.0 + age_days / stability_days.max(1.0))
    }

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

    fn compact_task_history(&self, history: &mut TaskVersionHistory) {
        let stability = 7.0;
        let keep = 3usize;
        if history.versions.len() <= keep { return; }

        let mut compacted: Vec<TaskVersion> = Vec::new();
        let split_point = history.versions.len().saturating_sub(keep);
        let recent = history.versions.split_off(split_point);

        #[allow(unused_variables)]
        let old_count = history.versions.len();
        let compacted_count = history.versions.iter()
            .filter(|v| self.fsrs_score(&v.timestamp, stability) < 0.15)
            .count();

        if compacted_count > 0 && compacted_count == old_count && old_count > 0 {
            compacted.push(TaskVersion {
                id: "v0-compacted".to_string(),
                version: 0,
                timestamp: history.versions.last().map(|v| v.timestamp.clone()).unwrap_or_default(),
                author: None,
                changes: vec![],
                compacted: true,
            });
        } else if compacted_count > 0 {
            for v in history.versions.drain(..) {
                if self.fsrs_score(&v.timestamp, stability) < 0.15 {
                    if !v.compacted {
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
    }

    pub fn rollback_task(&self, task_id: &str, target_version: u32, engine: &Arc<EngineState>) -> ToolResult<()> {
        let history = self.get_task_history(task_id)?;

        if history.versions.is_empty() {
            return Err(ToolError::not_found("version", &format!("task {}", task_id)));
        }

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
                    }
                }
            }
        }

        page::update_page(engine, task_id, &params)?;

        self.save_task_version_inner(task_id, rollback_changes, Some("rollback"))?;

        Ok(())
    }

    pub fn rollback_doc(&self, doc_id: &str, target_version: u32, engine: &Arc<EngineState>) -> ToolResult<()> {
        let history = self.get_doc_history(doc_id)?;

        if history.versions.is_empty() {
            return Err(ToolError::not_found("version", &format!("doc {}", doc_id)));
        }

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

        page::update_page(engine, doc_id, &params)?;

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
