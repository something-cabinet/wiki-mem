---
title: Search Scoring Formula
page_type: reference
status: reviewed
tags:
  - reference
  - search
  - scoring
  - BM25
---
# Search Scoring Formula

## BM25 Score (per document field)

```
BM25(doc, query) = Σ field_score × boost

field_score = weight × idf × ((tf × (k1 + 1)) / (tf + k1 × (1 - b + b × (field_len / avg_len))))

idf = 1.0 + ln((N - df + 0.5) / (df + 0.5))

where:
  tf     = term frequency in field
  df     = document frequency (docs containing term)
  N      = total documents in index
  k1     = 1.2 (BM25_K1)
  b      = 0.75 (BM25_B)
  weight = field weight (title: 4.0, body: 1.0)
```

Implemented in `Bm25Index::score_doc()` at `search/index.rs:122-152`.

## Rerank Boosts (additive, applied during search)

```
final_score = BM25_score + rerank_boost
```

| Condition | Boost |
|---|---|
| Title exactly matches query | +8.0 |
| Title starts with query | +4.0 |
| Title contains query | +2.0 |
| ID exactly matches query | +7.0 |
| Tag contains any query token | +3.0 |

Implemented in `rerank_boost()` at `search/index.rs:251-280`.

## Score Normalization

After BM25 scoring, all scores are normalized to `[0.01, 1.0]`:

```
normalized = max(0.01, min(1.0, round(score / max_score × 10000) / 10000))
```

Minimum floor of `0.01` ensures partial matches still rank above non-matches (which get score 0).

## Memory Salience Boost

Memory entries get a salience boost applied during result fusion:

```
boost = min(memory_salience_boost, memory_salience_clamp / score)
final_score = score × boost
```

Default config: `memory_salience_boost: 2.0`, `memory_salience_clamp: 0.1`

## Recency Boosts

### FSRS-6 Forgetting Curve (default)

```
w20 = 0.1542
factor = 0.9^(-1/w20) - 1
recency = (1 + factor × days_since_update / stability_days)^(-w20)
result = clamp(recency, 0.0, 1.0)
```

Default `stability_days: 7.0`.

Applied as a multiplier on task scores: `score = score × recency`

### Linear

```
recency = max(0, 1 - days_since_update / stability_days)
```

### Exponential

```
recency = exp(-days_since_update / stability_days)
```

## Hybrid Search (RRF Fusion)

When searching both pages and memory, results are merged via Reciprocal Rank Fusion:

```
rrf_score(id) = Σ[1 / (k + rank_of_id_in_type(type))]

where k = rrf_k (default 60)
```

Final results sorted by RRF score descending.

## Search Mode

| Mode | Page results | Memory results | Fusion |
|---|---|---|---|
| `keyword` | BM25 + rerank | BM25 (memory index) | RRF |
| `semantic` | Cosine similarity (embedding) | Cosine similarity | RRF |
| `hybrid` | BM25 + cosine via RRF | BM25 + cosine via score merge | RRF |

Semantic search disabled when no embedding model loaded (graceful fallback to keyword).

## Final Sort

After all scoring and fusion, results are sorted by:
1. Score descending
2. Centrality (incoming graph edges) descending
3. Page type rank descending (see table below)
4. Title ascending

## Total Score Pipeline

The final score for each result goes through this pipeline:

### Keyword path (pages)
```
raw = BM25(doc, query)                     → [0, ∞)
norm = normalize(raw)                      → [0, 1]
boosted = norm + rerank_boost(doc, query)  → [0, ~12]
final = normalize([boosted])               → [0.01, 1.0]
```

### Semantic path (pages)
```
query_vec = embed(query)
cosine = dot(query_vec, doc_vec) / (|query| × |doc|)  → [0, 1]
final = cosine                                          → [0, 1]
```

### Hybrid path (pages)
```
keyword_results   = BM25(doc, query)            → List<(id, score)>
semantic_results  = cos_sim(query, doc_vec)     → List<(id, score)>

for each id in keyword_results ∪ semantic_results:
    rrf_score = 1/(k + rank_in_keyword) + 1/(k + rank_in_semantic)
final = rrf_score                               → [0, ~2/k]
```

### Memory path (keyword)
```
raw = BM25(memory_entry, query)                → [0, ∞)
norm = normalize(raw)                          → [0.01, 1.0]
salience = min(salience_boost, clamp / norm)
final = norm × salience                        → [0.01, 2.0]
```

### Cross-type fusion
```
if both pages and memory results exist:
    for each id in all_results:
        rrf_score = 1/(k + rank_in_pages) + 1/(k + rank_in_memory)
    final = rrf_score
```

### Final sort
```
results.sort_by(|a, b| {
    b.score.cmp(a.score)                         # 1. score descending
    .then(b.centrality.cmp(a.centrality))        # 2. centrality descending
    .then(b.page_type_rank.cmp(a.page_type_rank)) # 3. type rank descending
    .then(a.title.cmp(b.title))                  # 4. title ascending
})
```

Where `page_type_rank` comes from `PageType::priority_rank()`:
task=7 → spec=6 → pattern=5 → concept=4 → decision=3 → howto=2 → reference=1 → note=0

| Page Type | Priority | Semantic | Example |
|-----------|----------|----------|---------|
| `task` | 7 | Actionable work unit | `fix-auth-timeout` |
| `spec` | 6 | Requirements specification | `user-auth` |
| `pattern` | 5 | Reusable solution | `arc-swap-graph` |
| `concept` | 4 | Domain explanation | `bm25-search` |
| `decision` | 3 | ADR lifecycle record | `axum-over-rocket` |
| `howto` | 2 | Step-by-step guide | `platform-setup` |
| `reference` | 1 | API/config reference | `scoring-config` |
| `note` | 0 | Informal content | `meeting-notes` |

## Constants Summary

| Constant | Value | Location |
|---|---|---|
| `BM25_K1` | 1.2 | `search/scoring.rs:4` |
| `BM25_B` | 0.75 | `search/scoring.rs:5` |
| `FSRS_W[20]` | 0.1542 | `search/scoring.rs:12` |
| `rrf_k` | 60 | config |
| `field_weights.title` | 4.0 | config |
| `field_weights.body` | 1.0 | config |
| `recency_model` | "fsrs" | config |
| `recency_stability_days` | 7 | config |
| `memory_salience_boost` | 2.0 | config |
| `memory_salience_clamp` | 0.1 | config |
