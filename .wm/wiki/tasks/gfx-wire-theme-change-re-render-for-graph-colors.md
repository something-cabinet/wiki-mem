---
title: GFX: Wire theme-change re-render for graph colors
type: task
status: todo
priority: high
tags: [spec:graph-ui-fix, theming]
---

Subscribe canvas directive and graph view to GraphColorService themeChanged event. On theme toggle: invalidate cached CSS colors, call render() on canvas, rebuild legend colors. Uses takeUntilDestroyed for cleanup.