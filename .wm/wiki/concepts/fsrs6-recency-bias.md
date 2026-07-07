---
title: FSRS-6 Recency Bias
page_type: concept
id: concepts/fsrs6-recency-bias
tags:
  - search
  - fsrs
  - recency
  - memory
  - scoring
---

# FSRS-6 Recency Bias

> Type: concept | Tags: [search, fsrs, recency, memory, scoring]

## Overview

Wiki Memory Engine uses the **FSRS-6 forgetting curve** (from the open-source Spaced Repetition algorithm) as a recency model for scoring search results and memory entries. This replaces simpler linear/exponential decay with a psychometrically validated model of human memory retention. FSRS-6 means the system naturally boosts recently-updated content without drowning out stable, important older documents.

## Technical Explanation

### Why FSRS Over Linear?

| Model | Day 0 | Day 7 (S=7) | Day 30 (S=7) | Day 90 (S=7) | Behavior |
|-------|-------|-------------|--------------|--------------|----------|
| Linear | 1.0 | 0.0 | 0.0 | 0.0 | Drops to zero at stability threshold |
| Exponential | 1.0 | ~0.368 | ~0.014 | ~0.000 | Vanishes quickly |
| **FSRS-6** | **1.0** | **~0.9** | **~0.78** | **~0.63** | Slow, realistic decay |
| None | 1.0 | 1.0 | 1.0 | 1.0 | No recency signal at all |

Linear decay is too aggressive — stable knowledge shouldn't hit zero. Exponential decay is too fast for long-lifespan information. FSRS provides a **psychologically accurate forgetting curve** where relevant-but-not-fresh content retains significant weight.

### FSRS-6 Formula

```
R(t) = (1 + factor × t / S)^(-w20)

where:
  factor = 0.9^(-1/w20) - 1
  w20 = W[20] = 0.1542 (from FSRS default parameters, stability decay)
  t    = days since last update
  S    = stability_days (configurable half-life, default 7)
```

The full FSRS-6 algorithm uses 21 parameters `W[0..20]`. WM uses the **default set** from `open-spaced-repetition/awesome-fsrs`:

```
W = [0.212, 1.2931, 2.3065, 8.2956, 6.4133, 0.8334, 3.0194, 0.001,
     1.8722, 0.1666, 0.796, 1.4835, 0.0614, 0.2629, 1.6483, 0.6014,
     1.8729, 0.5425, 0.0912, 0.0658, 0.1542]
```

Only `W[20]` (the stability decay parameter) is used in the simplified retrieval model. The full 21-parameter model would require spaced-repetition scheduling (review counts, difficulty tracking); WM uses only the retrieval-relevant decay curve.

### Implementation

```rust
pub fn recency_boost(days_since_update: f64, model: &str, stability_days: f64) -> f64 {
    match model {
        "fsrs" => {
            let w20 = FSRS_W[20];
            let factor = 0.9_f64.powf(-1.0 / w20) - 1.0;
            let r = (1.0 + factor * days_since_update / stability_days).powf(-w20);
            r.max(0.0).min(1.0)
        }
        "linear" => (1.0 - days_since_update / stability_days).max(0.0),
        "exponential" => (-days_since_update / stability_days).exp(),
        _ => 1.0, // "none"
    }
}
```

### Where It's Applied

The recency boost is applied **only to task pages** in `wm_search.query`. The logic:

1. For each task search result, compute `days_since_update` from the page's `updated_at` timestamp
2. Compute `recency = recency_boost(days_since_update, "fsrs", stability_days)`
3. Compute `salience` from the BM25 score itself (higher BM25 = more salient)
4. Combined boost: `cap_total_boost(recency, salience, max_boost)` where max_boost comes from config
5. Final score: `bm25_score × combined_boost`

Memory entries use a separate **salience boost** (not recency), controlled by `memory_salience_boost` and `memory_salience_clamp` in ScoringConfig.

### Boosting Cap

```rust
pub fn cap_total_boost(recency: f64, salience: f64, max_boost: f64) -> f64 {
    (recency * salience).min(max_boost)
}
```

The cap prevents a highly-recent, low-salience result from dominating a highly-relevant, slightly-older result purely on recency. The default `max_boost` is derived from the `memory_salience_clamp` config.

## Configuration Reference

```json
{
  "search": {
    "scoring": {
      "recency_model": "fsrs",
      "recency_stability_days": 7
    }
  }
}
```

| Parameter | Default | Description |
|-----------|---------|-------------|
| `recency_model` | `"fsrs"` | Decay model: `"fsrs"`, `"linear"`, `"exponential"`, `"none"` |
| `recency_stability_days` | `7` | Half-life in days. Higher = slower decay = recency matters less |

**When to tune:**
- Short-lived projects (rapid iteration): reduce `recency_stability_days` to 3-5
- Stable documentation repos: increase to 14-30 or use `"none"`
- Need aggressive freshness: use `"exponential"` with short stability

## Related Documents

- [BM25 Search Algorithm](./bm25-search.md) — the base scoring that recency boosts modify
- [ScoringConfig](./scoring-config.md) — full scoring parameter reference
- [Memory System](./memory-system.md) — how memory entries use salience instead of recency
