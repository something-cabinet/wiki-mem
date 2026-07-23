---
title: "Pattern: Field-Weighted BM25 Scoring"
type: pattern
tags: [search, bm25, scoring, field-weights]
status: reviewed
relates_to:
  - {type: references, target: wiki:patterns:code-aware-tokenizer}
  - {type: references, target: wiki:reference:scoring-config}
  - {type: references, target: wiki:reference:search-scoring-formula}
---

## When to use

Any search system where document fields carry different semantic importance. Title and tags are more descriptive than body text for relevance ranking. Standard BM25 treats all fields equally — field weighting corrects this by applying per-field boost factors during scoring.

## How it works

Field-weighted BM25 computes a separate BM25 score per field and combines them with configured weights:

```rust
fn field_weighted_score(query: &[String], doc: &Document, field_weights: &FieldWeights) -> f64 {
    let title_score = bm25_score(query, &doc.title) * field_weights.title;
    let body_score  = bm25_score(query, &doc.body)  * field_weights.body;
    let tags_score  = bm25_score(query, &doc.tags)  * field_weights.tags;
    title_score + body_score + tags_score
}
```

### Default Field Weights

| Field   | Weight | Rationale                                    |
|---------|--------|----------------------------------------------|
| Title   | 4.0    | Best single signal for page relevance         |
| Body    | 1.0    | Baseline content match                        |
| Tags    | 2.2    | Higher than body — tags are curated keywords  |

The weight imbalance means a title match dominates the score, which is intentional for search UIs where users typically query by topic name.

### Per-Type Weight Overrides

Page types can override default weights. For example, task pages might boost the `acceptance_criteria` section higher than general body text. The weight map is extensible via `config.json`:

```json
{
  "search": {
    "scoring": {
      "field_weights": {
        "title": 4.0,
        "body": 1.0,
        "tags": 2.2
      }
    }
  }
}
```

## Example

```
Query: "ArcSwap graph rebuild"

Title score:   0.142  × 4.0 = 0.568
Body score:    0.089  × 1.0 = 0.089
Tags score:    0.210  × 2.2 = 0.462
Total: 1.119
```

Without field weighting the total would be 0.441 — the tags contribution would be lost.

## Source

Derived from BM25 scoring implementation in the Wiki Memory Engine.
