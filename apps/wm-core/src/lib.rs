pub use wm_config as config;
pub mod engine;
pub mod graph;
pub mod mcp;
pub mod page;
pub mod reference_constant;
pub mod reference_service;
pub use reference_service as reference;
pub mod search;
pub mod skill;
pub mod task_board_service;
pub use task_board_service as task;
pub mod source_service;
pub use source_service as source;
pub mod version;
pub use wm_embed as embed;
pub use wm_error as error;
pub use wm_page_repo as page_repo;
pub use wm_status as status;
pub use wm_template_engine as template_engine;
pub use wm_util as util;
pub mod marker;
pub use wm_parser as parser;
pub use wm_vector_db as vector_db;

#[cfg(feature = "code-intel")]
pub use wm_code_intel as code_intel;
