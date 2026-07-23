---
title: GFX: Tune fjadra centering force for degree-0 nodes
type: task
status: todo
priority: medium
tags: [spec:graph-ui-fix, graph, layout]
---

Tune fjadra Center force strength so degree-0 nodes stay within viewport. Currently Center::new() with default strength may be too weak. Adjust parameters in the Rust layout command handler.