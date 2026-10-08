---
title: ArcSwap co-swap for graph + id_index
type: memory
tags: [graph, concurrency]
created_at: "2026-06-16T04:28:31.423Z"
updated_at: "2026-06-16T04:28:31.423Z"
---

schema_version: 1
state: |-
  Use ArcSwap<(StableGraph, HashMap)> not RwLock<DiGraph>. Build new graph in background, atomically swap. Co-swap id_index to prevent dangling NodeIndex references. Readers never block. Full reference: @doc/learnings/learning-wiki-mem-graph-architecture
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
