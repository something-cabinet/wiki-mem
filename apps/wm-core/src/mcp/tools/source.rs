use crate::mcp::prelude::*;
use serde_json::json;

use crate::source;

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
enum WmSourceAction {
    Add {
        #[schemars(description = "Path to the source file")]
        path: String,
    },
    Process {
        #[schemars(description = "Source ID to process")]
        id: String,
    },
    Complete {
        #[schemars(description = "Source ID")]
        id: String,
        #[schemars(description = "Page references created from this source")]
        page_refs: Option<Vec<String>>,
    },
    Error {
        #[schemars(description = "Source ID")]
        id: String,
        #[schemars(description = "Error message")]
        message: Option<String>,
    },
    List {
        #[schemars(description = "Filter by state: pending/processing/done/error/stale")]
        state: Option<String>,
    },
    Verify {
        #[schemars(description = "Source ID to verify")]
        id: String,
    },
    Discover {},
    Remove {
        #[schemars(description = "Source ID to remove")]
        id: String,
    },
    Status {
        #[schemars(description = "Source ID")]
        id: String,
    },
}

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    registry.register_typed(
        "wm_source",
        "Manage sources: add, list, status, discover, remove, process, complete, error, verify",
        move |input: WmSourceAction| match input {
            WmSourceAction::Add { path } => {
                let id = source::add_source(&engine, &path)?;
                Ok(json!({ "id": id, "state": "pending" }))
            }
            WmSourceAction::Process { id } => {
                let content = source::claim_source_and_read_content(&engine, &id)?;
                Ok(json!({ "id": id, "content": content }))
            }
            WmSourceAction::Complete { id, page_refs } => {
                let refs = page_refs.unwrap_or_default();
                source::complete_source(&engine, &id, &refs)?;
                Ok(json!({ "id": id, "status": "done", "pages": refs.len() }))
            }
            WmSourceAction::Error { id, message } => {
                let msg = message.unwrap_or_else(|| "Unknown error".into());
                source::error_source(&engine, &id, &msg)?;
                Ok(json!({ "id": id, "status": "error", "message": msg }))
            }
            WmSourceAction::List { state } => {
                let sources = source::list_sources(&engine, state.as_deref())?;
                Ok(json!({ "sources": sources, "total": sources.len() }))
            }
            WmSourceAction::Verify { id } => {
                let is_stale = source::verify_source(&engine, &id)?;
                Ok(json!({ "id": id, "stale": is_stale }))
            }
            WmSourceAction::Discover {} => {
                let (dirs, exts) = {
                    let config = engine
                        .config
                        .read()
                        .map_err(|_| ToolError::lock_poisoned("config"))?;
                    (config.source_dirs.clone(), config.source_extensions.clone())
                };
                let discovered = source::discover_sources(&engine, &dirs, Some(&exts))?;
                Ok(json!({ "discovered": discovered, "total": discovered.len() }))
            }
            WmSourceAction::Remove { id } => {
                source::remove_source(&engine, &id)?;
                Ok(json!({ "id": id, "status": "removed" }))
            }
            WmSourceAction::Status { id } => {
                let status = source::source_status(&engine, &id)?;
                Ok(status)
            }
        },
    );
}
