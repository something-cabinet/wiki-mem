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
```

### Real-world walkthrough

Say you search **"neural network"** across 100 docs. Here's how each term scores:

**"neural"** appears in 5 out of 100 docs:
- `df = 5`, `N = 100`
- `idf = 1.0 + ln((100 - 5 + 0.5) / (5 + 0.5))` ≈ **3.0`
- Rare term → high IDF → finding it is meaningful → big score contribution

**"network"** appears in 80 out of 100 docs:
- `df = 80`, `N = 100`
- `idf = 1.0 + ln((100 - 80 + 0.5) / (80 + 0.5))` ≈ **0.3**
- Common term → low IDF → finding it is expected → small score contribution

**Result**: "neural" does the heavy lifting. A doc that mentions "neural" ranks higher than one that only mentions "network", even if "network" appears more times. This is IDF at work — it rewards rare, specific terms over common, vague ones.

### Term frequency saturation

Now say a doc mentions "neural" 10 times (`tf = 10`), title weight is `4.0`, body weight is `1.0`:

- Title match: `score = 4.0 × 3.0 × ((10 × 1.2) / (10 + 1.2 × (1 - 0.75 + 0.75 × (5 / 10))))` ≈ **8.5**
- Body match: `score = 1.0 × 3.0 × ((10 × 1.2) / (10 + 1.2 × (1 - 0.75 + 0.75 × (200 / 150))))` ≈ **2.8**
- Title matters more (`4.0×` weight) and shorter fields saturate faster — finding a term in a title is more meaningful than finding it in a long body.

### Parameters

| Parameter | Typical value | What it controls |
|-----------|--------------|-----------------|
| k1 | 1.2 | Term frequency saturation — higher = TF matters more |
| b | 0.75 | Length normalization — higher = penalizes long docs more |
| title weight | 4.0 | How many times more important title matches are vs body |
| body weight | 1.0 | Baseline field weight

Implemented in `Bm25Index::score_doc()` at `search/index.rs:122-152`.

**What k1=1.2 does in practice:**
Times "neural" appears (tf) vs score contribution:
- 1 mention = 100%
- 5 mentions = ~305%
- 10 mentions = ~361%
- 100 mentions = ~405%

Mentioning a term 10 times is only ~3.6× as valuable as mentioning it once. This prevents keyword spam from dominating. Lower k1 (0.5) makes repetition matter less; higher k1 (2.0) makes it count more.

**What b=0.75 does:**
Doc A (100 words) vs Doc B (10,000 words), both mention "neural" 3 times. With b=0.75: Doc A scores much higher because "neural" is 3% of it vs 0.03% of Doc B. The term is clearly ABOUT Doc A. b=0 disables this entirely; b=1.0 penalizes proportionally to length.

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

**Example:** Searching "rust async":
- Exact match (+8.0): Title is literally "rust async"
- Starts with (+4.0): Title is "rust async patterns in actix"
- Contains (+2.0): Title is "understanding rust async"
- Not in title (0): Title is "tokio runtime internals"

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

**Walkthrough:**

| Raw BM25 score | clamp/score (0.1/score) | Effective boost min(2.0, ...) | Final score |
|---|---|---|---|
| 0.01 (barely matched) | 10.0 | 2.0 | 0.02 |
| 0.05 | 2.0 | 2.0 | 0.10 |
| 0.10 | 1.0 | 1.0 | 0.10 |
| 0.50 | 0.2 | 0.2 | 0.10 |
| 1.00 (strong match) | 0.1 | 0.1 | 0.10 |

Two effects: (1) Low-scoring memories get multiplied up to 2× — the `memory_salience_boost` amplifies weak matches. (2) High-scoring memories get capped at ~0.10 — the `clamp/score` term prevents any single memory from dominating results.

## Recency Boosts

### FSRS-6 Forgetting Curve (default)

```
w20 = 0.1542
factor = 0.9^(-1/w20) - 1
recency = (1 + factor × days_since_update / stability_days)^(-w20)
result = clamp(recency, 0.0, 1.0)
```

Default `stability_days: 7.0`.

**Walkthrough with stability_days: 7:**

| Days since update | Recency multiplier | Meaning |
|---|---|---|
| 0 (today) | 1.00 | Full relevance |
| 3 | ~0.92 | Barely faded |
| 7 | ~0.80 | Starting to age |
| 14 | ~0.64 | Noticeably stale |
| 30 | ~0.41 | Mostly forgotten |

The FSRS-6 curve models how human memory decays: the `w20` constant (0.1542) is a pre-trained parameter from spaced-repetition research. `stability_days` sets the "half-life" — after this many days, recency drops to roughly 0.8. The target retention probability is 0.9 (90% chance of recall at stability_days).

Applied as a multiplier on task scores: `score = score × recency`

### Linear

```
recency = max(0, 1 - days_since_update / stability_days)
```

**Walkthrough with stability_days=7:**
| Days old | Recency |
|---|---|
| 0 | 1.00 |
| 2 | 0.71 |
| 4 | 0.43 |
| 7 | 0.00 (stays 0) |

Linear decay is a straight line to zero. After stability_days, the task is completely stale.

### Exponential

```
recency = exp(-days_since_update / stability_days)
```

**Walkthrough with stability_days=7:**
| Days old | Recency |
|---|---|
| 0 | 1.00 |
| 7 | 0.37 |
| 14 | 0.14 |
| 30 | 0.01 |

Exponential decay never reaches zero — tasks fade but never disappear.

## Hybrid Search (RRF Fusion)

When searching both pages and memory, results are merged via Reciprocal Rank Fusion:

```
rrf_score(id) = Σ[1 / (k + rank_of_id_in_type(type))]

where k = rrf_k (default 60)
```

**Walkthrough with k=60:**

Say a doc ranks #1 in keyword search and #3 in semantic search:
- Keyword contribution: 1/(60+1) = 1/61 ≈ 0.0164
- Semantic contribution: 1/(60+3) = 1/63 ≈ 0.0159
- RRF score: 0.0164 + 0.0159 = 0.0323

Compare with a doc ranked #2 in both: 1/62 + 1/62 = 0.0323 — nearly identical. RRF with k=60 treats close ranks as roughly equal. Lower k (e.g., k=1) makes rank position matter much more: #1 vs #2 becomes 0.50 vs 0.33.

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
cosine_similarity(query, doc) = (query · doc) / (|query| × |doc|)

Think of the query and each document as arrows pointing in "meaning space." Cosine similarity measures the angle between them — it's the direction that matters, not the length. 1.0 = same direction (perfect match), 0.0 = perpendicular (unrelated). The denominator divides by length so that a long document and a short one are compared fairly.
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

Centrality counts how many other pages link TO this page. A page with 10 incoming links is probably more important than one with 0. It acts as a tiebreaker when search scores are equal — the more-connected page wins.

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

The priority reflects what users most want when searching: actionable work (task, 7) > requirements (spec, 6) > reusable knowledge (pattern, 5) > concepts (4) > decisions (3) > guides (howto, 2) > reference (1) > notes (0).

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
