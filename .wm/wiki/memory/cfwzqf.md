---
id: wiki:memory:cfwzqf
title: 'Cross-entity search: per-type BM25 + RRF + FSRS recency + IndexScheduler'
type: memory
tags: [search, architecture]
created_at: "2026-07-07T08:49:09.668Z"
updated_at: "2026-07-07T08:49:09.668Z"
---

WM's search uses per-type BM25 indexes (pages + memory) merged via RRF, not a unified index. FSRS-6 recency model for task ranking (defaults to fsrs, also supports linear/exponential/none). Debounced IndexScheduler replaces polling for rebuild triggers. Key gotcha: FSRS-6 R(t=S)=0.9, not 0.5. RRF must key by document ID, not position. Full reference: @doc/learnings/learning-cross-entity-search-per-type-bm25-fsrs-recency-debounced-indexscheduler