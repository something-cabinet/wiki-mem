---
title: GFX: Wire spacing slider to control all nodes (P3)
type: task
status: todo
priority: medium
tags: [spec:graph-ui-fix, graph, layout]
---

Wire the graph spacing slider to control fjadra global repulsion (ManyBody strength) instead of per-link distance. Add spacing field to ComputeLayoutPayload in Rust. Debounce slider changes and trigger recompute.