pub mod code_index_db;
pub mod engine_service;
pub mod graph_resolver;
pub mod ingest_service;
pub mod ts_config_resolver;

pub use code_index_db::CodeIndexDb;
pub use engine_service::{
    extract_deps, extract_edges, extract_symbols, infer_language_from_ext, load_lsp_config,
    CodeIntelEngine,
};
pub use graph_resolver::{resolve_code_edges, CodeEdgeGraph, CodeIndexSnapshot, ResolvedCodeEdge, detect_import_cycles};
pub use ingest_service::materialize_resolved_edges;
pub use ts_config_resolver::TsResolutionContext;
