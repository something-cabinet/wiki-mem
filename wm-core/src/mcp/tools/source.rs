use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::transport::ToolRegistry;
use crate::mcp::typed::TypedRegister;
use crate::source;

// ─── Input types ───────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
struct WmSourceAddInput {
    #[schemars(description = "Path to the source file")]
    path: String,
}

#[derive(Deserialize, JsonSchema)]
struct WmSourceProcessInput {
    #[schemars(description = "Source ID to process")]
    id: String,
}

#[derive(Deserialize, JsonSchema)]
struct WmSourceCompleteInput {
    #[schemars(description = "Source ID")]
    id: String,
    #[schemars(description = "Page references created from this source")]
    page_refs: Option<Vec<String>>,
}

#[derive(Deserialize, JsonSchema)]
struct WmSourceErrorInput {
    #[schemars(description = "Source ID")]
    id: String,
    #[schemars(description = "Error message")]
    message: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct WmSourceListInput {
    #[schemars(description = "Filter by state: pending/processing/done/error/stale")]
    state: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct WmSourceVerifyInput {
    #[schemars(description = "Source ID to verify")]
    id: String,
}

#[derive(Deserialize, JsonSchema)]
struct WmSourceDiscoverInput {}

#[derive(Deserialize, JsonSchema)]
struct WmSourceRemoveInput {
    #[schemars(description = "Source ID to remove")]
    id: String,
}

#[derive(Deserialize, JsonSchema)]
struct WmSourceStatusInput {
    #[schemars(description = "Source ID")]
    id: String,
}

/// Register source tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_write(
        "wm_source.add",
        "Add a raw source file to the registry",
        move |input: WmSourceAddInput| {
            let id = source::add_source(&e, &input.path)?;
            Ok(json!({ "id": id, "state": "pending" }))
        },
    );

    let e = engine.clone();
    registry.register_write(
        "wm_source.process",
        "Process a source (pending→processing)",
        move |input: WmSourceProcessInput| {
            let content = source::process_source(&e, &input.id)?;
            Ok(json!({ "id": input.id, "content": content }))
        },
    );

    let e = engine.clone();
    registry.register_write(
        "wm_source.complete",
        "Complete source processing (processing→done)",
        move |input: WmSourceCompleteInput| {
            let refs = input.page_refs.unwrap_or_default();
            source::complete_source(&e, &input.id, &refs)?;
            Ok(json!({ "id": input.id, "status": "done", "pages": refs.len() }))
        },
    );

    let e = engine.clone();
    registry.register_write(
        "wm_source.error",
        "Mark a source as errored",
        move |input: WmSourceErrorInput| {
            let msg = input.message.unwrap_or_else(|| "Unknown error".to_string());
            source::error_source(&e, &input.id, &msg)?;
            Ok(json!({ "id": input.id, "status": "error", "message": msg }))
        },
    );

    let e = engine.clone();
    registry.register_read(
        "wm_source.list",
        "List sources with optional state filter",
        move |input: WmSourceListInput| {
            let sources = source::list_sources(&e, input.state.as_deref())?;
            Ok(json!({ "sources": sources, "total": sources.len() }))
        },
    );

    let e = engine.clone();
    registry.register_write(
        "wm_source.verify",
        "Verify source staleness by hash",
        move |input: WmSourceVerifyInput| {
            let is_stale = source::verify_source(&e, &input.id)?;
            Ok(json!({ "id": input.id, "stale": is_stale }))
        },
    );

    let e = engine.clone();
    registry.register_write(
        "wm_source.discover",
        "Scan configured directories for new sources",
        move |_input: WmSourceDiscoverInput| {
            let (dirs, exts) = {
                let config = e.config.read().map_err(|_| ToolError::lock_poisoned("config"))?;
                (config.source_dirs.clone(), config.source_extensions.clone())
            };
            let discovered = source::discover_sources(&e, &dirs, Some(&exts))?;
            Ok(json!({ "discovered": discovered, "total": discovered.len() }))
        },
    );

    let e = engine.clone();
    registry.register_admin(
        "wm_source.remove",
        "Remove a source from the registry",
        move |input: WmSourceRemoveInput| {
            source::remove_source(&e, &input.id)?;
            Ok(json!({ "id": input.id, "status": "removed" }))
        },
    );

    let e = engine.clone();
    registry.register_read(
        "wm_source.status",
        "Get detailed source status",
        move |input: WmSourceStatusInput| {
            let status = source::source_status(&e, &input.id)?;
            Ok(status)
        },
    );
}
