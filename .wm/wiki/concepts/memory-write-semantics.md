---
title: Memory Write Semantics (from DeltaNet)
type: concept
id: wiki:concepts:memory-write-semantics
status: draft
tags: [memory, retrieval, write-semantics, ranking, deltanet]
---

# Memory Write Semantics (from DeltaNet)

> Type: concept | Tags: [memory, retrieval, write-semantics, ranking, deltanet]

## Overview

DeltaNet — *"Parallelizing Linear Transformers with the Delta Rule over Sequence Length"* (Yang, Wang, Zhang, Shen, Kim; NeurIPS 2024) — argues that **memory quality comes from write semantics, not capacity**. Additive accumulation (S += v⊗k) suffers interference: old and new information dilute each other, so in-context retrieval gets fuzzy. The delta rule restores explicit write semantics — retrieve old value, erase, write new value with a data-dependent write strength β — and it wins precisely on associative recall benchmarks (SWDE, FDA).

WM is symbolic, external, durable memory rather than a parametric neural state, so the paper's math (Householder products, chunk recurrences) does not transfer literally. The transferable parts are **principles**: erase-before-write, controlled write strength, compact on-demand materialization, and hybrid precision tiering. This page records those principles and their concrete implications for WM.

## The Delta Rule (reference)

Single-head state update (d×d state S_t):

```
v_old  = S_{t-1} k_t                 # retrieve old value
v_new  = β_t v_t + (1 − β_t) v_old   # interpolate with write strength
S_t    = S_{t-1} − v_old k_tᵀ + v_new k_tᵀ   # erase, then write
o_t    = S_t q_t                     # output
```

- β_t = σ(W_β x_t) ∈ (0,1) is a **data-dependent write strength**: β=1 fully overwrites, β=0 leaves memory unchanged.
- Additive (plain linear attention) is the β=1 case *without* the erase term — which is why old and new associations pile up and interfere.

## Mapping to WM

| DeltaNet principle | WM analogue | Status |
|---|---|---|
| Erase-before-write | Versioning + status lifecycle (draft → active → superseded/archived) | ✅ Exists |
| Write strength β | Confidence/authority weighting in retrieval | ⚠️ Gap |
| Compact state, lazy materialization | `wm_search.retrieve` token_budget; on-demand context assembly | ✅ Exists |
| Hybrid precision tiering | Exact `@wiki/ref` resolution + graph/LSP queries as a high-precision tier over BM25/embeddings | ⚠️ Partially implicit |

## Implications for WM

### 1. Confidence-weighted retrieval (β analogue)

Attach an **authority score** to pages that biases ranking when content conflicts: accepted ADRs and reviewed specs should dominate draft notes on the same subject. Today WM has statuses but no continuous, retrieval-influencing weight. Cheap to add — e.g., a per-status authority multiplier in `search.scoring` (like `memory_salience_boost`), applied before RRF fusion.

### 2. First-class supersedes-aware retrieval

`supersedes` already exists as an edge type, but search does not know about it. The delta-rule lesson: when a page is superseded, the old association must stop competing in retrieval. Make the index deprioritize (or exclude) pages that are the *target* of an active `supersedes` edge, so contradictory versions never surface at equal weight.

### 3. Associative recall eval for the wiki

The paper's core finding: additive memories fail at *specific* retrieval, not general similarity. WM should benchmark the same axis — a needle-in-haystack test over the wiki ("given this task, retrieve the exact spec/decision") to verify hybrid search surfaces the right *document*, not merely similar noise.

### 4. Hybrid precision tiering (architectural)

DeltaNet uses cheap linear layers everywhere but a *few* expensive global attention layers where recall precision matters. For WM: keyword/BM25 + embeddings for bulk queries; keep exact-reference resolution, graph traversal, and LSP/code queries as the never-fuzzed precision tier. Don't pay full semantic precision for every lookup.

## Where the analogy breaks

DeltaNet memory is **parametric, opaque, in-context** — a weight matrix rewritten by attending. WM memory is **symbolic, durable, external** — discrete documents with typed edges and full text. They are complementary layers of the agent memory stack: an agent still needs its own in-context memory to *use* what WM retrieves. Treat the paper as design inspiration, not a specification.

## Related Documents

- [Memory System](./memory-system.md) — WM memory layer, salience boost
- [Edge Types](./edge-types.md) — `supersedes` edge semantics
- [Cross-Entity Search](./cross-entity-search.md) — how page/memory types fuse in ranking
- [Graph Architecture](./graph-architecture.md) — typed graph traversal as precision tier