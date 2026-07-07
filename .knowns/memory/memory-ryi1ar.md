---
id: ryi1ar
title: ArcSwap co-swap for graph + id_index
layer: project
category: decision
tags:
  - graph
  - concurrency
createdAt: '2026-06-16T04:28:31.423Z'
updatedAt: '2026-06-16T04:28:31.423Z'
---

Use ArcSwap<(StableGraph, HashMap)> not RwLock<DiGraph>. Build new graph in background, atomically swap. Co-swap id_index to prevent dangling NodeIndex references. Readers never block. Full reference: @doc/learnings/learning-wiki-mem-graph-architecture
