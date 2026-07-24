---
id: wiki:concepts:bm25-search
title: BM25 Search Algorithm
type: concept
tags: [search, bm25, tokenizer, field-weights]
relates_to:
  - {type: references, target: wiki:patterns:field-weighted-bm25}
  - {type: references, target: wiki:patterns:code-aware-tokenizer}
  - {type: references, target: wiki:reference:scoring-config}
  - {type: references, target: wiki:reference:search-scoring-formula}
---
id: wiki:concepts:bm25-search

### IDF Formula

```rust
let idf = (1.0 + (total_docs - df + 0.5) / (df + 0.5)).ln();
```

This is the standard Robertson-Sparck Jones IDF variant with `ln()` smoothing (added 2026-07-24 — previously missing the `ln()`, making IDF linear in N/df).

### Code-Aware Tokenizer with Snowball Stemming

The tokenizer uses a three-pass approach specifically designed for technical content:

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

**Pass 3 — Snowball English stemming (rust-stemmers, Porter2):**
```
patterns → pattern     (plural → singular)
designer → design      (-er removal)
queries  → queri       (-ies → -i)
styling  → style       (-ing → ...)
```
Stemmed form is appended alongside the original when they differ. Applied symmetrically at both index time and query time. The `rust-stemmers` crate with `Algorithm::English` is used; the stemmer is lazily initialized via `LazyLock`.

The final token set includes **both** the full identifier and its components, plus stemmed variants. No global dedup — term frequencies accumulate naturally so BM25 TF saturation works correctly.