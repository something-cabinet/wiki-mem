---
title: Add touch event handlers to graph canvas directive
type: task
status: done
spec: specs/webgl-graph-rendering
priority: high
relates_to:
  - {type: implements, target: wiki:specs:webgl-graph-rendering}
---

In canvas-graph.directive.ts, add `touchstart`/`touchmove`/`touchend` handlers alongside existing mouse events to support touch-screen laptops and tablets.