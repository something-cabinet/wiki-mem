// ─── MCP Tool Prelude ──────────────────────────────────────────
// Common imports for all MCP tool handler modules.
// Each tool file can replace the standard 6-line import block with:
//   use crate::mcp::prelude::*;

pub use std::sync::Arc;

pub use schemars::JsonSchema;
pub use serde::Deserialize;

pub use crate::engine::EngineState;
pub use crate::error::ToolError;
pub use crate::mcp::transport::ToolRegistry;
