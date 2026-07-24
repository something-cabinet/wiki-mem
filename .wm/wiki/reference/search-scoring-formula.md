---
id: wiki:reference:search-scoring-formula
title: Search Scoring Formula
type: reference
tags: [reference, search, scoring, BM25]
status: reviewed
---
id: wiki:reference:search-scoring-formula

**Rerank boosts**: `rerank_boost()` and `post_rrf_rerank()` use raw string comparison for most checks, but for **exact match** they also compare Snowball-stemmed forms:

| Condition | Boost | Comparison |
|---|---|---|
| Title exactly matches query | +8.0 | raw OR stemmed match |
| Title starts with query | +4.0 | raw only |
| Query starts with title | +4.0 | raw only |
| Title contains query | +2.0 | raw only |

This means:
- **Exact match** (+8.0) fires for any morphological variant: `"design patterns" ↔ "Design Pattern"`, `"styling" ↔ "style"`, `"designer" ↔ "design"`
- **starts_with** (+4.0) works via raw string prefix matching in both directions
- Stemming for exact match uses the same Snowball stemmer as the tokenizer — applied symmetrically to both query and title