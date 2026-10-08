---
title: Graph edges_undirected helper (derived views)
type: memory
tags: [graph, derived-view, edge-reference]
status: active
---

schema_version: 1
state: |-
  Graph stores only stored edges; reciprocal/derived links are computed on demand. Use `graph.edges_undirected(idx) -> Vec<EdgeReference<GraphEdge>>` (apps/wm-core/src/graph/mod.rs) for bidirectional traversal + self-loop dedup, instead of manually iterating Outgoing+Incoming. Never store reciprocal edges in the graph — derive them at query time. See wiki:patterns:query-time-derived-views.
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
