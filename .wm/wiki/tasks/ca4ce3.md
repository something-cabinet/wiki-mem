---
title: 'GFX: Tune fjadra centering force for degree-0 nodes'
id: ca4ce3
type: task
status: done
priority: medium
tags:
- spec:graph-ui-fix
- graph
- layout
acceptance_criteria:
- text: Degree-0 nodes stay within the viewport after the fjadra layout settles
- text: Center force strength is tuned in the Rust layout command handler so the default (too-weak) strength is no longer used
---

Tune fjadra Center force strength so degree-0 nodes stay within viewport. Currently Center::new() with default strength may be too weak. Adjust parameters in the Rust layout command handler.