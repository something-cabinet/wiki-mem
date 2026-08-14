---
title: Graph edges_undirected helper (derived views)
type: memory
tags: [graph, derived-view, edge-reference]
status: active
---

Graph stores only stored edges; reciprocal/derived links are computed on demand. Use `graph.edges_undirected(idx) -> Vec<EdgeReference<GraphEdge>>` (apps/wm-core/src/graph/mod.rs) for bidirectional traversal + self-loop dedup, instead of manually iterating Outgoing+Incoming. Never store reciprocal edges in the graph — derive them at query time. See wiki:patterns:query-time-derived-views.