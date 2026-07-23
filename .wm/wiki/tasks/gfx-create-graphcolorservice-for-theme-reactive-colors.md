---
title: GFX: Create GraphColorService for theme-reactive colors
type: task
status: todo
priority: high
tags: [spec:graph-ui-fix, refactor, theming]
---

Create apps/wm-web/src/libs/graph/graph-color.service.ts with:
- nodeColor(type), nodeColorRGB(type), edgeColor(type), allPageTypes() methods
- Reads --page-type-{type} and --edge-type-{type} CSS vars
- MutationObserver on html.classList for theme change detection
- Emits signal/subject on theme toggle
- providedIn: root singleton. Required by Tasks 4,5,6,7,14.