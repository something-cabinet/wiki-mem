---
id: wiki:memory:cfwzqf
title: 'Cross-entity search: per-type BM25 + RRF + FSRS recency + IndexScheduler'
type: memory
tags: [search, architecture]
created_at: "2026-07-07T08:49:09.668Z"
updated_at: "2026-07-07T08:49:09.668Z"
---

schema_version: 1
state: |-
  WM's search uses per-type BM25 indexes (pages + memory) merged via RRF, not a unified index. FSRS-6 recency model for task ranking (defaults to fsrs, also supports linear/exponential/none). Debounced IndexScheduler replaces polling for rebuild triggers. Key gotcha: FSRS-6 R(t=S)=0.9, not 0.5. RRF must key by document ID, not position. Full reference: @doc/learnings/learning-cross-entity-search-per-type-bm25-fsrs-recency-debounced-indexscheduler
questions:
  - id: layer
    type: choice
    instructions: Which memory layer does this entry belong to?
    options:
    - project
    - global
    - session
  - id: store_or_skip
    type: noul
    instructions: This entry is worth storing as durable memory.
  - id: dedup_action
    type: choice
    instructions: How should this entry relate to existing memory?
    options:
    - new
    - merge
    - supersede
    - skip
  - id: confidence
    type: score
    instructions: How confident is the recorded knowledge?
    levels:
    - low
    - medium
    - high
answers: {}
