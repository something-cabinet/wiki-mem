---
title: Add golden-query eval harness then reduce ranking tiers
type: task
id: wiki:tasks:add-golden-query-eval-harness-then-reduce-ranking-tiers
status: todo
priority: medium
tags: [from-oracle, refactor, search, linus-remediation]
parent: wiki:tasks:apply-oracle-recommendations-from-linus-critique-review
acceptance_criteria:
  - text: "Golden-query eval harness: 30-50 queries with expected pages, recall@5 reported in CI (non-blocking at first)"
  - text: "Pre-normalize rerank (+8/+4/+2/+7/+3) deleted after measurement shows redundancy"
  - text: "enrich_search_results_from_graph deleted (search/mod.rs:6 export + query.rs:73-103); live comparator keeps its copied logic or calls shared helper"
  - text: "Ranking reduced from 8 tiers to: BM25 field weights, one post-normalization rerank, RRF for hybrid, recency for tasks, one deterministic tie-break"
  - text: "cargo build + clippy + search/e2e_search suites green"
implementation_notes: 'PARTIAL / split. LANDED + kept: golden_eval.rs harness (50 queries, #[ignore], recall@5=1.0 / recall@1=0.96 / mrr=0.9767); removal of the genuinely-dead enrich_search_results_from_graph; extraction of the single rank_cmp/RankKey comparator in query.rs. REVERTED: deleting the pre-normalize rerank boosts — the T3 review gate found P1 that this regressed the KEYWORD-ONLY exact-match path (post_rrf_rerank runs only on the hybrid+embedder branch; keyword/degraded/retrieve consume normalized BM25 by value and never enter RRF), and the multi-word golden set saturated at recall@5=1.0 so it could not detect the regression. This repo runs keyword-only, so the regression was live. Reverted packages/wm-search bm25_index_service.rs + field_model.rs + docs/search-scoring-formula.md to HEAD. Remaining ranking-reduction work (wire one rerank into keyword path behind exact-match golden queries, then remove boosts) moved to wiki:tasks:wire-single-rerank-into-keyword-search-path-then-remove-pre-normalize-boosts.'
---

From wiki:tasks:apply-oracle-recommendations-from-linus-critique-review AC-3. Oracle verdict LANDED: pre-normalize boosts (+8/+4/+2/+7/+3) implement the same intent as post-RRF boosts twice at two scales (bm25_index_service.rs:354-355 admits it); enrich_search_results_from_graph still exported and uncalled (search/mod.rs:6; query.rs:396 comment) — live wiring copied its comparator (query.rs:405-414 vs 93-102). Zero measurement infrastructure (no recall@k eval anywhere). Every constant is folklore. Tie-break tiers (centrality, page-type, id) are near-inert and harmless — keep as display metadata. Fix: build golden-query eval (30-50 queries with expected pages, recall@5 in CI, non-blocking first); then delete the pre-normalize rerank and the dead comparator, measuring each deletion. Gate: changes ranking behavior — needs its own review gate.