---
title: ScoringConfig
type: reference
tags: [config, scoring, tuning, search]
---

# ScoringConfig

> Type: concept | Tags: [config, scoring, tuning, search]

## Overview

`ScoringConfig` is the tunable scoring subsystem within `wm-core/src/config.rs`. It controls every numeric parameter that affects how WM ranks search results — field weights, recency modeling, graph traversal depths, memory salience boosts, debounce timing, and token budgets. All parameters have sensible defaults; tuning is optional but available for teams with specific relevance requirements.

## Full Parameter Reference

### Field Weights

```json
"field_weights": { "title": 4.0, "body": 1.0 }
```

| Field | Default | Role |
|-------|---------|------|
| `title` | `4.0` | Page title carries highest authority |
| `body` | `1.0` | Page body content, baseline weight |

Additional fields (`tags`, `id`) can be added to the map. Weight = 0 effectively removes that field from BM25 scoring.

**When to tune:** If your titles are auto-generated or noisy, reduce `title` to 2.0–3.0. If your body content is very structured, increase `body` to 1.5–2.0.

### Recency

```json
"recency_model": "fsrs",
"recency_stability_days": 7
```

| Parameter | Default | Options |
|-----------|---------|---------|
| `recency_model` | `"fsrs"` | `"fsrs"`, `"linear"`, `"exponential"`, `"none"` |
| `recency_stability_days` | `7` | 1–365 days |

Applied to task pages only. Controls how quickly old tasks lose relevance.

**When to tune:**
- Fast-moving sprints: `recency_stability_days: 3`, `recency_model: "linear"`
- Documentation-oriented repos: `recency_model: "none"`
- Mixed: keep `"fsrs"`, adjust `stability_days`

Effect of different values with FSRS:
| Days since update | stability=3 | stability=7 | stability=30 |
|---|---|---|---|
| 1 | 95% | 98% | 99% |
| 7 | 70% | 80% | 95% |
| 14 | 46% | 64% | 88% |
| 30 | 19% | 41% | 75% |

### Memory Salience

```json
"memory_salience_boost": 2.0,
"memory_salience_clamp": 0.1
```

| Parameter | Default | Description |
|-----------|---------|-------------|
| `memory_salience_boost` | `2.0` | Raw multiplier on memory entry BM25 scores |
| `memory_salience_clamp` | `0.1` | Minimum boost floor: `max(salience_boost, clamp/score)` |

Memory entries use salience boost (not recency) because they represent durable project knowledge. The clamp ensures even low-scoring memories get a minimum visibility floor.

**When to tune:**
- Memory entries drowning out wiki pages: reduce `memory_salience_boost` to 1.0–1.5
- Memory entries never appearing in results: increase `memory_salience_boost` to 3.0–4.0
- Want page/memory parity: set `memory_salience_boost` to 1.0, `memory_salience_clamp` to 0.0

### Graph Depth

```json
"graph_depth_rrf": 1,
"graph_depth_retrieve": 2,
"graph_depth_retrieve_min_priority": 5,
"graph_depth_neighbors_default": 2,
"graph_depth_neighbors_max": 5
```

| Parameter | Default | Range | Description |
|-----------|---------|-------|-------------|
| `graph_depth_rrf` | `1` | 1–3 | When merging search results with RRF, also include directly connected pages. Depth 1 = one hop (immediate neighbors). |
| `graph_depth_retrieve` | `2` | 1–5 | BFS (breadth-first search) depth when gathering context — explore all immediate neighbors before going deeper. |
| `graph_depth_retrieve_min_priority` | `5` | 0–10 | Only traverse edges with priority >= 5. Edge priorities range 0-10: 0 = weak tag links, 10 = hard dependencies. |
| `graph_depth_neighbors_default` | `2` | 1–5 | How many graph hops to expand when viewing a page's neighbors. |
| `graph_depth_neighbors_max` | `5` | 1–10 | Max allowed neighbors depth |

**When to tune `graph_depth_retrieve` and `graph_depth_retrieve_min_priority`:**
- Sparse graphs (few connections): increase depth to 3–4
- Dense graphs (many connections): reduce depth to 1, raise min_priority to 7
- Token budget hitting limit: reduce depth first before reducing budget

### Debounce & Budget

```json
"debounce_ms": 500,
"retrieve_token_budget": 2048
```

| Parameter | Default | Range | Description |
|-----------|---------|-------|-------------|
| `debounce_ms` | `500` | 100–5000 | Debounce period in ms before index rebuild triggers. Every edit makes the index stale, but rebuilding immediately on every edit is wasteful. WM waits this long after the LAST edit before rebuilding. At 500ms: 3 saves in 2 seconds triggers one rebuild. |
| `retrieve_token_budget` | `2048` | 256–131072 | Max tokens for context assembly. Tokens roughly = 3/4 of a word. When WM assembles page content for an AI, it truncates at this limit. 2048 tokens ≈ 1500 words. |

**When to tune `debounce_ms`:**
- Rapid page creation (agent workflows): increase to 1000–2000ms to batch more writes
- Interactive editing: reduce to 200–300ms for faster feedback
- CI/CD pipelines: set to 0 (no debounce), rebuild explicitly

**When to tune `retrieve_token_budget`:**
- Working with models that have large context windows (128K+): increase to 8192–16384
- CLI-only consumption: keep at 2048
- Embedding-rich context needs: increase to 4096

### Search Defaults (parent SearchConfig)

These are siblings of `scoring` in `SearchConfig`:

```json
"search": {
  "default_mode": "hybrid",
  "default_limit": 20,
  "rrf_k": 60,
  "scoring": { ... }
}
```

| Parameter | Default | Description |
|-----------|---------|-------------|
| `default_mode` | `"hybrid"` | `"keyword"`, `"semantic"`, `"hybrid"` |
| `default_limit` | `20` | Results per query |
| `rrf_k` | `60` | RRF dampening constant (higher = ranks matter less, consensus matters more; lower = being #1 is heavily rewarded) |

## Complete Configuration Example

```json
{
  "search": {
    "default_mode": "hybrid",
    "default_limit": 20,
    "rrf_k": 60,
    "scoring": {
      "field_weights": {
        "title": 4.0,
        "body": 1.0
      },
      "recency_model": "fsrs",
      "recency_stability_days": 7,
      "memory_salience_boost": 2.0,
      "memory_salience_clamp": 0.1,
      "graph_depth_rrf": 1,
      "graph_depth_retrieve": 2,
      "graph_depth_retrieve_min_priority": 5,
      "graph_depth_neighbors_default": 2,
      "graph_depth_neighbors_max": 5,
      "debounce_ms": 500,
      "retrieve_token_budget": 2048
    }
  }
}
```

## Related Documents

- [BM25 Search Algorithm](./bm25-search.md) — how field weights feed into BM25
- [FSRS-6 Recency Bias](./fsrs6-recency-bias.md) — how recency_model works
- [Graph Edge Types and Traversal](./graph-edge-types-traversal.md) — graph depth parameters
- [Memory System](./memory-system.md) — memory salience in context
- [Cross-Entity Search](./cross-entity-search.md) — RRF fusion constant