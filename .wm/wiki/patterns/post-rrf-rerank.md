---
title: Pattern: Post-RRF Rerank for Hybrid Search
type: pattern
---

---
title: Post-RRF Rerank for Hybrid Search
type: pattern
---

## Problem
BM25 rerank boosts (exact title +8, ID match +7, tag +3) were applied before normalization and RRF fusion. Normalization compressed them, RRF discarded raw scores entirely — boosts had zero effect in hybrid mode.

## Solution
Move rerank boosts to AFTER RRF fusion as a separate post-processing step. Apply Knowns-inspired heuristics on the fused scores where boosts actually take effect:

- Title density: +0.03 per query token found in title
- Exact title match: +0.15 additive
- Tag overlap: proportional (matched/total × 0.1 × score)
- Exact ID match: +0.10 additive

## When to Use
Any hybrid search system using RRF fusion where you want rerank signals to actually affect the final ranking.

## When Not to Use
If you don't use RRF or if scores aren't normalized (inline boosts work fine).

## Related
- @task:gfx-wire-spacing-slider-to-control-all-nodes-p3
