---
title: Reinforce derived-state indexing flow
type: memory
tags: [indexing, retrieval, freshness, search-quality]
status: active
---

Indexing reinforcement: freshness = one refresh_all_if_stale (graph+sections+BM25) used by search/validate/graph, never clear stale after a partial heal; index structured frontmatter VALUES not keys; embedding integrity validates persisted model/version on load and clears on mismatch with an actionable degraded reason; incremental single-page graph update; golden eval de-ignored. Full: @doc/patterns/reinforce-indexing-flow