use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
struct InputDoc {
    id: String,
    title: String,
    #[serde(default)]
    body: String,
}

#[derive(Serialize)]
struct ScoredDoc {
    id: String,
    score: f64,
}

const K1: f64 = 1.2;
const B: f64 = 0.75;

/// Simple BM25 re-ranker — takes a query and document list, returns scored results.
#[wasm_bindgen]
pub fn rerank(query: &str, docs_json: &str) -> Result<String, JsValue> {
    let docs: Vec<InputDoc> = serde_json::from_str(docs_json).map_err(|e| e.to_string())?;
    let query_tokens: Vec<&str> = query.split_whitespace().collect();

    if query_tokens.is_empty() || docs.is_empty() {
        return Ok(serde_json::to_string(&Vec::<ScoredDoc>::new()).unwrap());
    }

    let n = docs.len() as f64;

    // Compute document frequencies and average doc length
    let mut df: HashMap<String, f64> = HashMap::new();
    let mut total_len = 0.0;
    let mut doc_lengths: Vec<f64> = Vec::new();

    for doc in &docs {
        let text = format!("{} {}", doc.title, doc.body);
        let tokens: Vec<&str> = text.split_whitespace().collect();
        let len = tokens.len() as f64;
        doc_lengths.push(len);
        total_len += len;

        let mut seen = std::collections::HashSet::new();
        for token in &tokens {
            if seen.insert(*token) {
                *df.entry(token.to_string()).or_insert(0.0) += 1.0;
            }
        }
    }

    let avgdl = if n > 0.0 { total_len / n } else { 1.0 };

    // Score each document
    let mut scored: Vec<ScoredDoc> = Vec::new();

    for (i, doc) in docs.iter().enumerate() {
        let text = format!("{} {}", doc.title, doc.body);
        let tokens: Vec<&str> = text.split_whitespace().collect();
        let dl = doc_lengths[i];

        let mut score = 0.0;
        for qt in &query_tokens {
            let q = *qt;
            let tf = tokens.iter().filter(|t| **t == q).count() as f64;
            if tf == 0.0 {
                continue;
            }

            let idf =
                ((n - df.get(q).copied().unwrap_or(0.0) + 0.5) / (df.get(q).copied().unwrap_or(0.0) + 0.5) + 1.0)
                    .ln();
            let bm25 = idf * ((tf * (K1 + 1.0)) / (tf + K1 * (1.0 - B + B * dl / avgdl)));
            score += bm25;
        }

        // Title boost: exact title match gets +0.15
        if doc.title.to_lowercase().contains(&query.to_lowercase()) {
            score += 0.15;
        }

        scored.push(ScoredDoc {
            id: doc.id.clone(),
            score,
        });
    }

    // Sort by score descending
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    Ok(serde_json::to_string(&scored).unwrap())
}
