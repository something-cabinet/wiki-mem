pub mod memory;
pub mod query;
pub mod retrieve;

pub use query::{
    merge_results_by_rrf, run_unified_search, QueryParams, QueryResult, SearchResponse,
};
pub use retrieve::context;
pub use wm_search::{
    cap_total_boost, recency_boost, tokenize, Bm25Index, Field, IndexedDoc, SearchResult,
};

use crate::engine::SectionDoc;
use crate::engine::EngineState;

pub fn indexed_doc_from_section(s: &SectionDoc) -> IndexedDoc {
    IndexedDoc {
        id: s.section_id.clone(),
        fields: vec![
            Field::new("header", &s.header, 4.0),
            Field::new("body", &s.body, 1.0),
            Field::new("id", &s.section_id, 0.0),
            Field::new("title", &s.title, 0.0),
            Field::new("tags", &s.tags.join(" "), 0.0),
        ],
    }
}

pub fn embedding_degraded_reason(engine: &EngineState) -> Option<String> {
    if !engine.embedder.is_loaded() {
        let model = engine.embedder.model_name();
        return Some(format!(
            "Semantic search unavailable — embedding model '{model}' not loaded. Run `wm model download {model}`."
        ));
    }
    engine.vector_store.load_note()
}

#[cfg(test)]
mod tests;
