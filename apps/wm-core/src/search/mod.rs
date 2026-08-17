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

/// Convert a `SectionDoc` into an `IndexedDoc` with field weights matching the
/// index schema. Used by all BM25 rebuild sites so field weights stay in sync.
///
/// Field weights: header=4.0, body=1.0, id/title/tags=0.0 (title/tags checked
/// by `post_rrf_rerank` string matching, not BM25 scoring weight).
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

#[cfg(test)]
mod tests;
