/// A search result with normalized score
#[derive(Clone, Debug)]
pub struct SearchResult {
    pub id: String,
    pub score: f64,
    pub snippet: String,
    pub page_type_rank: u8, // populated externally: task=7, spec=6, decision=5, concept=4, pattern=3, howto=2, reference=1
    pub centrality: usize,  // populated externally: inbound edge count
}
