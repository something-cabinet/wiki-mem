use std::sync::Arc;

use crate::engine::EngineState;
use crate::mcp::handler::ToolArgs;
use crate::mcp::transport::ToolRegistry;
use crate::page;

/// Register task tool handlers
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_with_desc(
        "wm_task.check_ac",
        "Check an acceptance criterion",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let ac_indices = args.optional_string_array("criteria");
            let indices: Vec<u64> = ac_indices.iter().filter_map(|s| s.parse().ok()).collect();
            let update = serde_json::json!({ "checked_ac": indices });
            page::update_page(&e, &id, &update)?;
            Ok(serde_json::json!({ "id": id, "checked": indices }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc(
        "wm_task.uncheck_ac",
        "Uncheck an acceptance criterion",
        Arc::new(move |params| {
            let args = ToolArgs::new(params);
            let id = args.require_string("id")?;
            let ac_indices = args.optional_string_array("criteria");
            let indices: Vec<u64> = ac_indices.iter().filter_map(|s| s.parse().ok()).collect();
            let update = serde_json::json!({ "unchecked_ac": indices });
            page::update_page(&e, &id, &update)?;
            Ok(serde_json::json!({ "id": id, "unchecked": indices }))
        }),
    );

    let e = engine.clone();
    registry.register_with_desc("wm_task.board", "Task board grouped by status", Arc::new(move |_params| {
        let board = crate::task::task_board(&e);
        Ok(serde_json::json!(board))
    }));
}
