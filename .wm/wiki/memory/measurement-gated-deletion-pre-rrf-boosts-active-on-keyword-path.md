---
title: Measurement-gated deletion — pre-RRF boosts active on keyword path
type: memory
tags: [search, ranking, measurement]
status: active
---

Never delete scoring/ranking code without measuring recall@k per search path first. Pre-RRF boosts are inert for hybrid paths (rank-order only) but ACTIVE for keyword-only paths (score by value). A saturated golden eval (all multi-word queries) can miss exact-match regressions — add exact-title/id queries. Full reference: @wiki/patterns/measurement-gated-deletion