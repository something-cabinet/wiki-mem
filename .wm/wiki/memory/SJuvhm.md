---
title: Global OnceLock for axum state workaround
type: memory
tags: [axum, rust, state]
created_at: "2026-07-14T04:41:47.530Z"
updated_at: "2026-07-14T04:41:47.530Z"
---

schema_version: 1
state: |-
  When axum's Router<S> type constraints fight you (into_make_service not found), use a global OnceLock<Arc<T>> for complex state instead of putting it in AppState. Only for single-instance, once-initialized state. Full reference: @doc/learnings/proxy-architecture-single-entrypoint
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
