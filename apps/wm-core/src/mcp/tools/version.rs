use crate::mcp::prelude::*;
use serde::Serialize;
use crate::version::{DocVersionHistory, TaskVersionHistory, VersionStore};

// ─── wm_version.list ──────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
struct WmVersionListInput {
    #[schemars(description = "Entity type: 'task' or 'doc'")]
    entity_type: String,
    #[schemars(description = "Entity ID (task ID or doc wiki ID)")]
    entity_id: String,
}

#[derive(Serialize)]
struct WmVersionListOutput {
    entity_id: String,
    entity_type: String,
    current_version: u32,
    versions: Vec<serde_json::Value>,
    total: usize,
}

// ─── wm_version.get ───────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
struct WmVersionGetInput {
    #[schemars(description = "Entity type: 'task' or 'doc'")]
    entity_type: String,
    #[schemars(description = "Entity ID (task ID or doc wiki ID)")]
    entity_id: String,
    #[schemars(description = "Version ID (e.g. 'v1', 'v2')")]
    version_id: String,
}

#[derive(Serialize)]
struct WmVersionGetOutput {
    entity_id: String,
    entity_type: String,
    version: serde_json::Value,
}

// ─── wm_version.rollback ──────────────────────────────────

#[derive(Deserialize, JsonSchema)]
struct WmVersionRollbackInput {
    #[schemars(description = "Entity type: 'task' or 'doc'")]
    entity_type: String,
    #[schemars(description = "Entity ID (task ID or doc wiki ID)")]
    entity_id: String,
    #[schemars(description = "Version ID to rollback to (e.g. 'v1', 'v2')")]
    version_id: String,
}

#[derive(Serialize)]
struct WmVersionRollbackOutput {
    entity_id: String,
    entity_type: String,
    rolled_back_to: String,
    changes_applied: usize,
    status: String,
}

// ─── Tool Registration ──────────────────────────────────────

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_typed(
        "wm_version.list",
        "List all versions for a task or doc entity",
        move |input: WmVersionListInput| -> Result<serde_json::Value, ToolError> {
            let root = e
                .project_root
                .read()
                .map_err(|_| ToolError::lock_poisoned("project_root"))?
                .clone();
            let store = VersionStore::new(root.join(".wm"));

            match input.entity_type.as_str() {
                "task" => {
                    let history: TaskVersionHistory = store.get_task_history(&input.entity_id)?;
                    let versions: Vec<serde_json::Value> = history
                        .versions
                        .iter()
                        .map(|v| {
                            serde_json::json!({
                                "id": v.id,
                                "version": v.version,
                                "timestamp": v.timestamp,
                                "changes": v.changes,
                                "compacted": v.compacted,
                            })
                        })
                        .collect();
                    let total = versions.len();
                    Ok(serde_json::to_value(WmVersionListOutput {
                        entity_id: input.entity_id,
                        entity_type: "task".to_string(),
                        current_version: history.current_version,
                        versions,
                        total,
                    })
                    .unwrap_or(serde_json::Value::Null))
                }
                "doc" => {
                    let history: DocVersionHistory = store.get_doc_history(&input.entity_id)?;
                    let versions: Vec<serde_json::Value> = history
                        .versions
                        .iter()
                        .map(|v| {
                            serde_json::json!({
                                "id": v.id,
                                "version": v.version,
                                "timestamp": v.timestamp,
                                "changes": v.changes,
                                "path": v.path,
                                "compacted": v.compacted,
                            })
                        })
                        .collect();
                    let total = versions.len();
                    Ok(serde_json::to_value(WmVersionListOutput {
                        entity_id: input.entity_id,
                        entity_type: "doc".to_string(),
                        current_version: history.current_version,
                        versions,
                        total,
                    })
                    .unwrap_or(serde_json::Value::Null))
                }
                other => Err(ToolError::invalid_params(format!(
                    "Unknown entity_type '{}'. Valid: task, doc",
                    other
                ))),
            }
        },
    );

    let e = engine.clone();
    registry.register_typed(
        "wm_version.get",
        "Get details for a specific version of a task or doc entity",
        move |input: WmVersionGetInput| -> Result<serde_json::Value, ToolError> {
            let root = e
                .project_root
                .read()
                .map_err(|_| ToolError::lock_poisoned("project_root"))?
                .clone();
            let store = VersionStore::new(root.join(".wm"));

            match input.entity_type.as_str() {
                "task" => {
                    let history = store.get_task_history(&input.entity_id)?;
                    let version = history
                        .versions
                        .iter()
                        .find(|v| v.id == input.version_id)
                        .ok_or_else(|| {
                            ToolError::not_found(
                                "version",
                                &format!("{} in task {}", input.version_id, input.entity_id),
                            )
                        })?;
                    Ok(serde_json::to_value(WmVersionGetOutput {
                        entity_id: input.entity_id,
                        entity_type: "task".to_string(),
                        version: serde_json::to_value(version)
                            .map_err(|e| ToolError::serde_error("serialize version", e))?,
                    })
                    .unwrap_or(serde_json::Value::Null))
                }
                "doc" => {
                    let history = store.get_doc_history(&input.entity_id)?;
                    let version = history
                        .versions
                        .iter()
                        .find(|v| v.id == input.version_id)
                        .ok_or_else(|| {
                            ToolError::not_found(
                                "version",
                                &format!("{} in doc {}", input.version_id, input.entity_id),
                            )
                        })?;
                    Ok(serde_json::to_value(WmVersionGetOutput {
                        entity_id: input.entity_id,
                        entity_type: "doc".to_string(),
                        version: serde_json::to_value(version)
                            .map_err(|e| ToolError::serde_error("serialize version", e))?,
                    })
                    .unwrap_or(serde_json::Value::Null))
                }
                other => Err(ToolError::invalid_params(format!(
                    "Unknown entity_type '{}'. Valid: task, doc",
                    other
                ))),
            }
        },
    );

    registry.register_typed(
        "wm_version.rollback",
        "Rollback a task or doc entity to a previous version by applying inverse changes",
        move |input: WmVersionRollbackInput| -> Result<serde_json::Value, ToolError> {
            let root = engine
                .project_root
                .read()
                .map_err(|_| ToolError::lock_poisoned("project_root"))?
                .clone();
            let store = VersionStore::new(root.join(".wm"));

            match input.entity_type.as_str() {
                "task" => rollback_task(&engine, &store, &input.entity_id, &input.version_id),
                "doc" => rollback_doc(&engine, &store, &input.entity_id, &input.version_id),
                other => Err(ToolError::invalid_params(format!(
                    "Unknown entity_type '{}'. Valid: task, doc",
                    other
                ))),
            }
        },
    );
}

// ─── Rollback logic ────────────────────────────────────────

/// Parse version number from a version ID string like "v1" → 1
fn parse_version_number(version_id: &str) -> Result<u32, ToolError> {
    let num_str = version_id.trim_start_matches('v');
    num_str
        .parse::<u32>()
        .map_err(|_| ToolError::invalid_params(format!("Invalid version ID '{}'", version_id)))
}

/// Rollback a task to a previous version.
fn rollback_task(
    engine: &Arc<EngineState>,
    store: &VersionStore,
    entity_id: &str,
    version_id: &str,
) -> Result<serde_json::Value, ToolError> {
    let target_version = parse_version_number(version_id)?;
    let history = store.get_task_history(entity_id)?;

    // Look up the target version entry to count changes
    let target_entry = history
        .versions
        .iter()
        .find(|v| v.version == target_version)
        .ok_or_else(|| ToolError::not_found("version", &format!("{} in task {}", version_id, entity_id)))?;

    if target_entry.changes.is_empty() {
        return Err(ToolError::invalid_params(format!(
            "Version {} has no changes to rollback",
            version_id
        )));
    }

    store.rollback_task(entity_id, target_version, engine)?;

    Ok(serde_json::to_value(WmVersionRollbackOutput {
        entity_id: entity_id.to_string(),
        entity_type: "task".to_string(),
        rolled_back_to: version_id.to_string(),
        changes_applied: target_entry.changes.len(),
        status: "rolled_back".to_string(),
    })
    .unwrap_or(serde_json::Value::Null))
}

/// Rollback a doc to a previous version.
fn rollback_doc(
    engine: &Arc<EngineState>,
    store: &VersionStore,
    entity_id: &str,
    version_id: &str,
) -> Result<serde_json::Value, ToolError> {
    let target_version = parse_version_number(version_id)?;
    let history = store.get_doc_history(entity_id)?;

    let target_entry = history
        .versions
        .iter()
        .find(|v| v.version == target_version)
        .ok_or_else(|| ToolError::not_found("version", &format!("{} in doc {}", version_id, entity_id)))?;

    if target_entry.changes.is_empty() {
        return Err(ToolError::invalid_params(format!(
            "Version {} has no changes to rollback",
            version_id
        )));
    }

    store.rollback_doc(entity_id, target_version, engine)?;

    Ok(serde_json::to_value(WmVersionRollbackOutput {
        entity_id: entity_id.to_string(),
        entity_type: "doc".to_string(),
        rolled_back_to: version_id.to_string(),
        changes_applied: target_entry.changes.len(),
        status: "rolled_back".to_string(),
    })
    .unwrap_or(serde_json::Value::Null))
}
