---
title: Replace hardcoded graph node colors with theme-aware values
type: task
status: todo
spec: specs/webgl-graph-rendering
priority: medium
---

In canvas-graph.directive.ts, replace hardcoded hex node colors with values from CSS custom properties, and change white stroke to use `var(--ring)` for reliable cross-theme appearance.