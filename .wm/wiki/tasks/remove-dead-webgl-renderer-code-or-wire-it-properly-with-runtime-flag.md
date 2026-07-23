---
title: Remove dead WebGL renderer code or wire it properly with runtime flag
type: task
status: todo
priority: medium
tags: [graph, cleanup, webgl]
---

From @designer review: WebglGraphRenderer is unreachable — useWebgl defaults false, the 500-node check runs at AfterViewInit when array is always empty. 580 lines of regl shaders sit unused. Either: (1) evaluate WebGL at data-arrival time with a proper flag, or (2) delete the dead code until the >500-node use case is real. Also instantiate the declared-but-never-used ResizeObserver.