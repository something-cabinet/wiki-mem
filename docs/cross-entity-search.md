# Cross-Entity Search

> Type: concept | Tags: [search, cross-entity, rrf, memory, pages]

## Overview

Cross-entity search allows a single `wm_search.query` call to search across **wiki pages** AND **memory entries** simultaneously, with type-level filtering and RRF (Reciprocal Rank Fusion) to merge heterogeneous result sets into a single ranked list. This is one of WM's differentiating features over Knowns, which maintains separate per-type stores.

## Technical Explanation

### The Two-Index Architecture

WM maintains **two separate BM25 indexes** behind `ArcSwap`:

| Index | Location | Source | ID Format |
|-------|----------|--------|-----------|
| Page BM25 | `EngineState.bm25_index` | `.wm/wiki/**/*.md` sections | `wiki:type:slug` |
| Memory BM25 | `EngineState.memory_index` | `.wm/memory/*.json` entries | `memory:<id>` |

Both indexes use the same `Bm25Index` struct with the same scoring algorithm — only the field weights differ (memory entries use title + content + tags fields weighted identically to pages).

### Type Filter

The `type` parameter in `wm_search.query` controls which indexes to query:

| `type` Value | Pages | Memory | Use Case |
|-------------|-------|--------|----------|
| `"all"` | ✓ | ✓ | Full project context search |
| `"page"` | ✓ | ✗ | Wiki-only queries |
| `"task"` | ✓ (filtered) | ✗ | Task-specific results |
| `"memory"` | ✗ | ✓ | Retrieve stored patterns/decisions |

When `type` is `"all"` or `"memory"`, both indexes are queried and results are merged. When `type` is `"page"` or `"task"`, only the page index is searched (with additional page-type filtering for `"task"`).

### RRF Fusion Across Entity Types

For `"all"` searches in hybrid mode, results from the page BM25 and memory BM25 are merged using **Reciprocal Rank Fusion**:

$$ \text{RRF}(d) = \frac{1}{k + r_{\text{pages}}(d)} + \frac{1}{k + r_{\text{memory}}(d)} $$

Where $k = \text{rrf\_k}$ (default 60, from `config.json` → `search.rrf_k`). The $k=60$ constant dampens rank differences — a #1 result gets ~0.0164, a #100 result gets ~0.00625. This prevents a single #1 result from dominating the merged list.

### Three Search Modes

| Mode | Pages | Memory | Fallback |
|------|-------|--------|----------|
| `keyword` | BM25 only | BM25 only | N/A |
| `semantic` | Cosine similarity | Not supported (memories don't have vectors) | Error if no model |
| `hybrid` | BM25 + cosine → RRF | BM25 only | Falls back to BM25 if no model |

In hybrid mode with `type="all"`:
1. Page results: RRF fusion of BM25 + semantic cosine (if model loaded)
2. Memory results: BM25 only (memories are not embedded)
3. Both result sets merged with post-processing enrichment

### Cross-Entity Scoring Adjustments

Memory entries get a **salience boost** applied on top of their BM25 score:

```rust
let adjusted_score = memory_score.max(memory_salience_boost.min(memory_salience_clamp / memory_score));
```

This ensures memory entries remain visible even when competing against higher-scoring wiki pages. The boost is controlled by:
- `memory_salience_boost` (default 2.0) — raw multiplier
- `memory_salience_clamp` (default 0.1) — floor guarantee

### Type Enrichment

Results are enriched with `type` field (`"page"` or `"memory"`) and page results additionally get `page_type` (task/spec/concept/pattern/decision/howto/reference) from the graph snapshot. This allows consumers to filter/display results by entity type.

### Per-Type Indexes (vs Knowns)

**Knowns:** Maintains separate per-type indexes — tasks, docs, memories, decisions each have their own store with different schemas and search APIs.

**WM:** Unifies everything under two BM25 indexes (pages + memory) with shared scoring and type-filter at query time. The advantage: a single `wm_search.query` call with `type="all"` returns a ranked, merged list. The tradeoff: less specialized per-type ranking (mitigated by page_type_rank in stable sort).

## Configuration Reference

```json
{
  "search": {
    "default_mode": "hybrid",
    "rrf_k": 60,
    "scoring": {
      "memory_salience_boost": 2.0,
      "memory_salience_clamp": 0.1
    }
  }
}
```

## Related Documents

- [BM25 Search Algorithm](./bm25-search.md) — the underlying BM25 scoring
- [Memory System](./memory-system.md) — MemoryEntry format and indexing
- [ScoringConfig](./scoring-config.md) — memory salience parameters
