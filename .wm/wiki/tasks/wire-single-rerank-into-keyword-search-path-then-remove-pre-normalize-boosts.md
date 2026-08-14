---
title: Wire single rerank into keyword search path then remove pre-normalize boosts
type: task
id: "wiki:tasks:wire-single-rerank-into-keyword-search-path-then-remove-pre-normalize-boosts"
status: todo
priority: medium
tags: [from-review, linus-remediation, search, ranking, deferred-from-t3]
acceptance_criteria:
  - text: "Add exact-title, exact-id, starts-with and tag-match queries to the golden_eval harness (apps/wm-core/tests/golden_eval.rs) that exercise exact-match promotion, and confirm they RED when pre-normalize boosts are absent from the keyword path"
  - text: "Wire post_rrf_rerank (or an equivalent single rerank) into the keyword and no-embedder degraded-hybrid and retrieve paths in apps/wm-core/src/search/query.rs so exact-match promotion applies on every path, verifying score-scale compatibility with normalized BM25"
  - text: "Only after the exact-match golden queries stay GREEN through the wiring, remove the pre-normalize rerank_boost from packages/wm-search bm25_index_service.rs, achieving a single post-normalization rerank tier"
  - text: "clippy -D warnings clean; wm-core lib + wm-search + e2e_search + golden_eval green"
---

Deferred from wiki:tasks:add-golden-query-eval-harness-then-reduce-ranking-tiers (T3). The T3 review gate found a P1: deleting the pre-normalize rerank boosts (+8/+4/+2/+7/+3) regressed the KEYWORD-ONLY path. post_rrf_rerank runs only on the hybrid+embedder branch (apps/wm-core/src/search/query.rs:229); keyword mode (:157), no-embedder degraded-hybrid (:207) and retrieve.rs consume normalized BM25 score by value and never enter RRF, so the pre-normalize boosts were the only exact-match/exact-id/tag promotion there. Deleting them reverted the prior deliberate fix (.slim/deepwork/bm25-stemming-and-rerank-fix.md Phase 2). This repo runs keyword-only (no embedding model loaded), so the regression is live. The boost deletion was REVERTED in the remediation pass to restore correct behavior; the eval harness and the dead enrich_search_results_from_graph removal were KEPT.

The correct consolidation (single post-normalization rerank tier for ALL paths) requires wiring the rerank into the keyword/degraded/retrieve paths FIRST, gated by exact-match golden queries (the current 50-query golden set is all multi-word natural language and saturates at recall@5=1.0, so it cannot detect exact-match tie-break regressions). Do this behind those new golden queries, verify score-scale compatibility, then remove the pre-normalize boosts.