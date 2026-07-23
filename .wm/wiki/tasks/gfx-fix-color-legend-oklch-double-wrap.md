---
title: GFX: Fix color legend oklch double-wrap
type: task
status: todo
priority: high
tags: [spec:graph-ui-fix, bug]
---

buildPageTypes() in graph-view.component.ts double-wraps oklch(oklch(...)) making legend swatches transparent. Fix: read --page-type-{key} directly, do NOT re-wrap in oklch(). Use GraphColorService if available.