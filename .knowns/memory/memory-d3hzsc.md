---
id: d3hzsc
title: Rayon parallelization pattern for WM — par_iter map + sequential merge
layer: project
category: pattern
tags:
  - rayon
  - parallelism
  - pattern
  - search
  - graph
createdAt: '2026-07-09T17:41:38.474Z'
updatedAt: '2026-07-09T17:41:38.474Z'
---

Rayon (rayon = "1") was added to parallelize CPU-bound operations across wm-core. Two patterns used:

1. **Read-only .par_iter().map()** — for operations where each element is scored/computed independently with no shared state (Bm25Index::search(), top_k_cosine()). One-liner change: `.iter()` → `.par_iter()`.

2. **Parallel map + sequential merge** — for operations that accumulate into shared maps (Bm25Index::build(), build_embeddings phases 1&3). Parallel compute per-element into owned partials, then sequential merge.

3. **Collect paths + parallel read/parse** — for mixed I/O+CPU operations (build_sections_from_wiki, build_graph_from_wiki, rebuild_memory_index_from_dir). Walkdir collects paths sequentially (fast), then par_iter().map() reads + processes each file in parallel.

**Key constraint:** petgraph add_node/add_edge is not thread-safe, so graph construction stays sequential after parallel file parsing.

**Files changed:** wm-core/src/search.rs, embed.rs, graph.rs; wm-core/Cargo.toml, wm-cli/Cargo.toml. All 148 tests pass.
