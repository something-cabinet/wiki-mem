---
title: GFX: Verify + fix graph loading/error/empty states
type: task
status: todo
priority: medium
tags: [spec:graph-ui-fix, ux]
---

Verify graph loading/error/empty states work with fjadra-only pipeline:
- Loading: already works (spinner)
- Error: computeLayout error handler doesn't set error state — fix it
- Empty: already works (no graph data message)
- After d3-force removal, ensure browser dev mode still works