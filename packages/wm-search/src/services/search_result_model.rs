/// A search result with normalized score
#[derive(Debug)]
pub struct SearchResult {
    pub id: String,
    pub score: f64,
    pub snippet: String,
    pub page_type_rank: u8,
    pub centrality: usize,
}

/// Score breakdown for a search result — shows how each rerank heuristic
/// contributed to the final score.
#[derive(Clone, Debug, serde::Serialize)]
pub struct ScoreBreakdown {
    pub bm25: f64,
    pub rrf: f64,
    pub semantic: f64,
    pub title_density: f64,
    pub exact_title: f64,
    pub title_starts_with: f64,
    pub title_contains: f64,
    pub tag_overlap: f64,
    pub exact_id: f64,
    pub recency: f64,
    pub final_score: f64,
}
