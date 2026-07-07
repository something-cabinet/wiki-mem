---
title: BM25 Search Algorithm
page_type: concept
id: concepts/bm25-search
tags:
  - search
  - bm25
  - tokenizer
  - field-weights
---

# BM25 Search Algorithm

> Type: concept | Tags: [search, bm25, tokenizer, field-weights]

## Overview

Wiki Memory Engine implements a custom BM25 (Best Match 25) search index for keyword-based retrieval across wiki pages and memory entries. Unlike dependency-heavy approaches that pull in the `bm25` crate, WM builds its own ~300-line implementation in `wm-core/src/search.rs` with field-weighted scoring, a code-aware tokenizer, and rerank boosts — all within a single `Bm25Index` struct.

## Technical Explanation

### Core BM25 Formula

The standard BM25 score for a document `d` given query `q` is:

```
score(d, q) = Σ IDF(q_i) × (tf × (k1 + 1)) / (tf + k1 × (1 - b + b × (|d| / avgdl)))
```

Where:
- **k1 = 1.2** — controls term frequency saturation
- **b = 0.75** — controls length normalization (0 = no normalization, 1 = full)
- **tf** — term frequency in the field
- **|d|** — field length in tokens
- **avgdl** — average field length across all documents

WM extends this with **field-weighted scoring**:

```
score(d, q) = Σ field_weight × IDF(q_i) × (tf × (k1+1)) / denom
```

### IDF Formula

```rust
let idf = 1.0 + (total_docs - df + 0.5) / (df + 0.5);
```

This is the Robertson-Sparck Jones IDF variant with smoothing, preventing division by zero.

### Field Weights

| Field | Default Weight | Purpose |
|-------|---------------|---------|
| `title` | 4.0 | Exact page name carries highest auth |
| `id` | 3.0 | Path-based ID provides structural signal |
| `tags` | 2.2 | Curated metadata, stronger than body |
| `body` | 1.0 | Full content, baseline weight |

These are configurable via `config.json` → `search.scoring.field_weights` (a `HashMap<String, f64>`).

### Code-Aware Tokenizer

The tokenizer uses a two-pass approach specifically designed for technical content:

**Pass 1 — Full identifiers:**
```
ERR_AUTH_401 → "err_auth_401"
client-secret  → "client-secret"
```
Preserves the compound identifier as a single token for exact matches.

**Pass 2 — Sub-tokenization:**
```
err_auth_401 → ["err", "auth", "401"]
client-secret → ["client", "secret"]
```
Splits on `_` and `-` boundaries so partial matches still work.

The final token set includes **both** the full identifier and its components. This means searching `auth` matches both `ERR_AUTH_401` and standalone usage. Searching `ERR_AUTH_401` exactly matches the full identifier with higher TF weight.

Implementation uses a `LazyLock<Regex>` for efficiency:
```rust
static TOKEN_RE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"[a-z0-9_\-]+").unwrap());
```

### Rerank Boosts

After BM25 scoring, additional boosts refine ranking:

| Match Type | Boost | Description |
|-----------|-------|-------------|
| Exact title | +8.0 | Query exactly equals page title |
| Title prefix | +4.0 | Title starts with query |
| Title contains | +2.0 | Title contains query string |
| Exact ID | +7.0 | Query exactly equals page ID |
| Tag match | +3.0 | Any query token appears in tags |

### Score Normalization

```
normalized_score = raw / maxScore
```
- Floor at 0.01 for any result with score > 0
- Clamped to [0.0, 1.0]
- Rounded to 4 decimal places

### Zero-Result Guard

Results with BM25 score of 0 are filtered out entirely. This prevents a gibberish query like `xyznonexistent123!!!` from returning every document.

### Stable Sort Order

Results are sorted by:
1. Score descending
2. Graph centrality descending (inbound edges)
3. Page type rank: Task(7) > Spec(6) > Decision(5) > Concept(4) > Pattern(3) > Howto(2) > Reference(1)
4. Page ID alphabetically

### Index Lifecycle

The `Bm25Index` holds:
- `docs: Vec<IndexedDoc>` — all indexed documents with their fields and pre-computed tokens
- `total_docs: usize` — for IDF computation
- `term_freq: HashMap<String, usize>` — document frequency per term
- `field_lengths: HashMap<String, usize>` — total token count per field type
- `field_doc_counts: HashMap<String, usize>` — number of docs containing each field type

The index is rebuilt on every `index.rebuild` and stored behind `ArcSwap<Bm25Index>` for lock-free reads. Rebuilds target <50ms for 500 sections.

## Configuration Reference

```json
// .wm/config.json → search.scoring
{
  "search": {
    "default_mode": "hybrid",
    "default_limit": 20,
    "rrf_k": 60,
    "scoring": {
      "field_weights": {
        "title": 4.0,
        "body": 1.0
      }
    }
  }
}
```

To add custom field weights (e.g., `tags`, `id`), extend the `field_weights` map. These override the hardcoded defaults.

## Related Documents

- [FSRS-6 Recency Bias](./fsrs6-recency-bias.md) — how recency boosts interact with BM25 scores
- [ScoringConfig](./scoring-config.md) — all configurable scoring parameters
- [Cross-Entity Search](./cross-entity-search.md) — how BM25 spans pages + memory
