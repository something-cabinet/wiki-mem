pub mod agent_hooks;
pub mod config;
pub mod decision;
pub mod embed_files;
pub mod engine;
pub mod error;
pub mod graph;
pub mod mcp;
pub use mcp::transport::ToolRegistry;
pub mod memory_gate;
pub mod page;
pub mod page_repo;
pub mod platform_service;
pub mod reference_constant;
pub mod reference_service;
pub use reference_service as reference;
pub mod search;
pub mod shared;
pub mod skill;
pub mod status;
pub mod source_service;
pub use source_service as source;

pub mod util;
pub mod version;
pub use wm_embed as embed;

#[cfg(feature = "code-intel")]
pub use wm_code_intel as code_intel;
