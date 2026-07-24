---
id: wiki:patterns:field-weighted-bm25
title: Pattern: Field-Weighted BM25 Scoring
type: pattern
tags: [search, bm25, scoring, field-weights]
status: reviewed
relates_to:
  - {type: references, target: wiki:patterns:code-aware-tokenizer}
  - {type: references, target: wiki:reference:scoring-config}
  - {type: references, target: wiki:reference:search-scoring-formula}
---
id: wiki:patterns:field-weighted-bm25

## When to use

Any search system where document fields carry different semantic importance. Title and tags are more descriptive than body text for relevance ranking. Standard BM25 treats all fields equally — field weighting corrects this by applying per-field boost factors during scoring.

## How it works

Field-weighted BM25 computes a separate BM25 score per field and combines them with configured weights. The IDF formula uses the standard Robertson-Sparck Jones variant with `ln()` smoothing (added 2026-07-24):

```rust
let idf = (1.0 + (total_docs - df + 0.5) / (df + 0.5)).ln();
```

Then:

```rust
fn field_weighted_score(query: &[String], doc: &Document, field_weights: &FieldWeights) -> f64 {
    let title_score = bm25_score(query, &doc.title) * field_weights.title;
    let body_score  = bm25_score(query, &doc.body)  * field_weights.body;
    let tags_score  = bm25_score(query, &doc.tags)  * field_weights.tags;
    title_score + body_score + tags_score
}
```

For rerank boosts and post-RRF hybrid boost details, see @wiki/reference:search-scoring-formula and @wiki/patterns:post-rrf-rerank.