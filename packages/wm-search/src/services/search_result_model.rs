/// A search result with normalized score
#[derive(Debug)]
pub struct SearchResult {
    pub id: String,
    pub score: f64,
    pub snippet: String,
    pub page_type_rank: u8, // populated externally: task=7, spec=6, decision=5, concept=4, pattern=3, howto=2, reference=1
    pub centrality: usize,  // populated externally: inbound edge count
}

/// Score breakdown for a search result — shows how each rerank heuristic
/// contributed to the final score.
#[derive(Clone, Debug, serde::Serialize)]
pub struct ScoreBreakdown {
    pub bm25: f64,          // Raw BM25 score (normalized, 0 if keyword-only)
    pub rrf: f64,           // RRF fusion score (0 if keyword-only)
    pub semantic: f64,      // Semantic cosine score (0 if keyword-only)
    pub title_density: f64, // Title density bonus added
    pub exact_title: f64,   // Exact title match bonus added
    pub tag_overlap: f64,   // Tag overlap bonus added
    pub exact_id: f64,      // Exact ID match bonus added
    pub recency: f64,       // FSRS-6 recency multiplier
    pub final_score: f64,   // Final score after all boosts
}
