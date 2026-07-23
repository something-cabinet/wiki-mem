---
title: Replace hardcoded graph edge colors with CSS custom properties
type: task
status: todo
spec: specs/webgl-graph-rendering
priority: medium
relates_to:
  - {type: implements, target: wiki:specs:webgl-graph-rendering}
---

In canvas-graph.directive.ts, replace hardcoded rgba values like `rgba(156,163,175,0.4)` with values read from CSS custom properties so edges adapt to the current theme.