use std::fmt;

/// Search mode for query execution
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchMode {
    Keyword,
    Semantic,
    Hybrid,
}

impl SearchMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "semantic" => SearchMode::Semantic,
            "hybrid" => SearchMode::Hybrid,
            _ => SearchMode::Keyword,
        }
    }

    /// Auto-detect: code identifiers → keyword, natural language → hybrid
    pub fn auto_detect(query: &str) -> Self {
        // Check for code identifiers (ERR_, snake_case, kebab-case)
        let has_code_pattern = query.contains('_')
            || query.contains('-')
            || query.chars().filter(|c| c.is_uppercase()).count() > 2;
        if has_code_pattern {
            SearchMode::Keyword
        } else {
            SearchMode::Hybrid
        }
    }
}

impl fmt::Display for SearchMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchMode::Keyword => write!(f, "keyword"),
            SearchMode::Semantic => write!(f, "semantic"),
            SearchMode::Hybrid => write!(f, "hybrid"),
        }
    }
}

/// Embedder trait — allows pluggable ONNX or no-op implementations
pub trait Embedder: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>, String>;
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, String>;
    fn is_loaded(&self) -> bool;
    fn dimensions(&self) -> usize;
    fn model_name(&self) -> &str;
}

/// No-op embedder — returns empty vectors, used when no ONNX model available
pub struct NoopEmbedder;

impl Embedder for NoopEmbedder {
    fn embed(&self, _text: &str) -> Result<Vec<f32>, String> {
        Err("No embedding model loaded. Use 'wm model download' first.".to_string())
    }

    fn embed_batch(&self, _texts: &[&str]) -> Result<Vec<Vec<f32>>, String> {
        Err("No embedding model loaded. Use 'wm model download' first.".to_string())
    }

    fn is_loaded(&self) -> bool { false }
    fn dimensions(&self) -> usize { 0 }
    fn model_name(&self) -> &str { "none" }
}

/// Compute cosine similarity between two vectors
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 { return 0.0; }
    (dot / (norm_a * norm_b)) as f64
}

/// RRF fusion: combine BM25 and semantic rankings
pub fn rrf_fusion(
    bm25_results: &[(String, f64)],  // (id, score)
    semantic_results: &[(String, f64)], // (id, score)
    k: f64,
) -> Vec<(String, f64)> {
    use std::collections::BTreeMap;

    let mut scores: BTreeMap<String, f64> = BTreeMap::new();

    for (i, (id, _)) in bm25_results.iter().enumerate() {
        let rank = i as f64 + 1.0;
        *scores.entry(id.clone()).or_insert(0.0) += 1.0 / (k + rank);
    }

    for (i, (id, _)) in semantic_results.iter().enumerate() {
        let rank = i as f64 + 1.0;
        *scores.entry(id.clone()).or_insert(0.0) += 1.0 / (k + rank);
    }

    let mut result: Vec<(String, f64)> = scores.into_iter().collect();
    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_mode_auto_detect() {
        assert_eq!(SearchMode::auto_detect("ERR_AUTH_401"), SearchMode::Keyword);
        assert_eq!(SearchMode::auto_detect("auth-service"), SearchMode::Keyword);
        assert_eq!(SearchMode::auto_detect("how does authentication work"), SearchMode::Hybrid);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_rrf_fusion() {
        // b is rank 1 in both → clear winner
        let bm25 = vec![("b".into(), 0.9), ("a".into(), 0.5), ("c".into(), 0.3)];
        let sem  = vec![("b".into(), 0.8), ("c".into(), 0.6), ("a".into(), 0.3)];
        let fused = rrf_fusion(&bm25, &sem, 60.0);
        assert_eq!(fused.len(), 3);
        assert_eq!(fused[0].0, "b", "b is rank 1 in both lists");
    }

    #[test]
    fn test_noop_embedder() {
        let e = NoopEmbedder;
        assert!(!e.is_loaded());
        assert_eq!(e.dimensions(), 0);
        assert!(e.embed("test").is_err());
    }
}
