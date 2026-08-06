---
title: "GFX: Fix color legend oklch double-wrap"
id: b25ada
type: task
status: todo
priority: high
tags: [spec:graph-ui-fix, bug]
acceptance_criteria:
  - text: "buildPageTypes() reads --page-type-{key} CSS vars directly without re-wrapping values in oklch()"
  - text: "Legend swatches render with visible (non-transparent) colors"
  - text: "GraphColorService is used for color derivation where available"
---

buildPageTypes() in graph-view.component.ts double-wraps oklch(oklch(...)) making legend swatches transparent. Fix: read --page-type-{key} directly, do NOT re-wrap in oklch(). Use GraphColorService if available.
