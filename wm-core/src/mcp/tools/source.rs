use std::sync::Arc;

use serde_json::json;

use crate::engine::EngineState;
use crate::error::ToolError;
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;
use crate::source;

/// Register source tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_with_schema(
        "source.add",
        "Add a raw source file to the registry",
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to the source file" }
            },
            "required": ["path"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let path = args.require_string("path")?;
            let id = source::add_source(&e, &path)?;
            Ok(serde_json::json!({ "id": id, "state": "pending" }))
        }),
    );

    let e = engine.clone();
    registry.register_with_schema(
        "source.process",
        "Process a source (pending→processing)",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Source ID to process" }
            },
            "required": ["id"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let content = source::process_source(&e, &id)?;
            Ok(serde_json::json!({ "id": id, "content": content }))
        }),
    );

    let e = engine.clone();
    registry.register_with_schema(
        "source.complete",
        "Complete source processing (processing→done)",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Source ID" },
                "page_refs": { "type": "array", "items": { "type": "string" }, "description": "Page references created from this source" }
            },
            "required": ["id"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let refs = args.optional_string_array("page_refs");
            source::complete_source(&e, &id, &refs)?;
            Ok(serde_json::json!({ "id": id, "status": "done", "pages": refs.len() }))
        }),
    );

    let e = engine.clone();
    registry.register_with_schema(
        "source.error",
        "Mark a source as errored",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Source ID" },
                "message": { "type": "string", "description": "Error message" }
            },
            "required": ["id"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let msg = args
                .optional_string("message")
                .unwrap_or_else(|| "Unknown error".to_string());
            source::error_source(&e, &id, &msg)?;
            Ok(serde_json::json!({ "id": id, "status": "error", "message": msg }))
        }),
    );

    let e = engine.clone();
    registry.register_with_schema(
        "source.list",
        "List sources with optional state filter",
        json!({
            "type": "object",
            "properties": {
                "state": { "type": "string", "description": "Filter by state: pending/processing/done/error/stale" }
            }
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let state = args.optional_string("state");
            let sources = source::list_sources(&e, state.as_deref())?;
            Ok(serde_json::json!({ "sources": sources, "total": sources.len() }))
        }),
    );

    let e = engine.clone();
    registry.register_with_schema(
        "source.verify",
        "Verify source staleness by hash",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Source ID to verify" }
            },
            "required": ["id"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let is_stale = source::verify_source(&e, &id)?;
            Ok(serde_json::json!({ "id": id, "stale": is_stale }))
        }),
    );

    let e = engine.clone();
    registry.register_with_schema(
        "source.discover",
        "Scan configured directories for new sources",
        json!({
            "type": "object",
            "properties": {}
        }),
        Arc::new(move |_params| {
            let (dirs, exts) = {
                let config = e.config.read().map_err(|_| ToolError::lock_poisoned("config"))?;
                (config.source_dirs.clone(), config.source_extensions.clone())
            };
            let discovered = source::discover_sources(&e, &dirs, Some(&exts))?;
            Ok(serde_json::json!({ "discovered": discovered, "total": discovered.len() }))
        }),
    );

    let e = engine.clone();
    registry.register_with_schema(
        "source.remove",
        "Remove a source from the registry",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Source ID to remove" }
            },
            "required": ["id"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            source::remove_source(&e, &id)?;
            Ok(serde_json::json!({ "id": id, "status": "removed" }))
        }),
    );

    let e = engine.clone();
    registry.register_with_schema(
        "source.status",
        "Get detailed source status",
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Source ID" }
            },
            "required": ["id"]
        }),
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let status = source::source_status(&e, &id)?;
            Ok(status)
        }),
    );
}
