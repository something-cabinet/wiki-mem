---
title: Measurement-Gated Deletion for Multi-Path Scoring Code
type: pattern
id: wiki:patterns:measurement-gated-deletion
status: draft
tags: [pattern, search, ranking, testing, measurement]
relates_to:
  - {type: references, target: wiki:tasks:add-golden-query-eval-harness-then-reduce-ranking-tiers}
---

## Problem

Deleting "redundant" code (rerank boosts, dead exports, duplicate logic) can silently regress behavior when the code is only redundant on ONE path but active on another. Multi-path systems (keyword vs hybrid vs semantic) make it hard to reason about which code is load-bearing where.

## Solution

Before deleting any scoring/ranking code:

1. Build a golden-query eval harness FIRST (recall@k, mrr) with queries that exercise EACH path the code touches (not just the most common path).
2. Run the harness, record a BASELINE.
3. Delete ONE thing at a time, re-measure, compare.
4. If recall regresses: STOP, revert that deletion, create a task to fix the gap first.
5. If recall is unchanged: keep the deletion, record the measurement as evidence.

The critical failure mode: a 50-query natural-language eval set saturated at recall@5=1.0 — it couldn't detect that exact-match/exact-id promotion was lost on the keyword-only path because no query exercised exact-match tie-breaking. **Query diversity across ALL code paths is essential.**

## When to Use

- Removing any additive scoring boost, rerank step, or post-processing in search/ranking pipelines.
- Consolidating duplicate logic where both copies might serve different callers.
- Any deletion where "this is redundant" is the justification — prove it with measurement, path by path.

## When Not to Use

- Deleting genuinely dead code (zero callers confirmed by grep + compiler) — no eval needed, just delete.
- Removing a feature entirely (not claiming redundancy, just sunsetting).

## Key Insight

Pre-RRF boosts (+8/+4/+2/+7/+3) are inert for the HYBRID path (RRF consumes only rank order, not score magnitude) but ACTIVE for the keyword-only path (scores consumed by value). The WM conventions doc says "boosts applied before RRF fusion are silently discarded" — this is true only for paths that actually enter RRF. Keyword-only mode never enters RRF.

## Related

- @wiki/tasks/add-golden-query-eval-harness-then-reduce-ranking-tiers
- @wiki/tasks/wire-single-rerank-into-keyword-search-path-then-remove-pre-normalize-boosts