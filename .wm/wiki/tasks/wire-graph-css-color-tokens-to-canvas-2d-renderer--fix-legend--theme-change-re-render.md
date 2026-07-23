---
title: Wire graph CSS color tokens to Canvas 2D renderer + fix legend + theme-change re-render
type: task
status: todo
priority: high
tags: [graph, theming, web-ui, ux]
---

From @designer review H2: (1) nodeColor() maps 4 types to same neutral gray (invisible in dark mode) instead of --page-type-* tokens. (2) Legend double-wraps oklch(oklch(...)) = transparent swatches. (3) Colors go stale on theme toggle. Fix: point both at cssColor('--page-type-' + type) (WebGL renderer already does this), pass raw var without re-wrapping, re-render on theme change. Subsumes tasks replace-hardcoded-graph-node-colors and replace-hardcoded-graph-edge-colors.