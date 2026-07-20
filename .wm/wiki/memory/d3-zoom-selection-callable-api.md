---
title: d3-zoom callable API expects Selection, not raw element
type: memory
---

---
title: "d3-zoom callable API expects Selection, not raw element"
type: memory
status: active
tags: [d3, d3-zoom, angular, graph, canvas]
---

## Problem

Using `d3-zoom`'s `ZoomBehavior` as a callable function with a raw `HTMLCanvasElement`:

```typescript
const zoom = d3Zoom<HTMLCanvasElement, unknown>()
  .scaleExtent([0.1, 4])
  .on('zoom', (event) => { ... });

zoom(this.canvas as any, () => this.canvas as any);
```

Causes: `TypeError: selection2.property is not a function`

## Root Cause

In d3 v7, `ZoomBehavior` is callable but expects a d3 `Selection` as its argument, not a raw DOM element. Internally, d3-zoom calls `selection.property("__zoom", zoomIdentity)` which fails when `selection` is not a proper d3 Selection object.

The second argument `() => this.canvas` is also incorrect — there is no two-argument callable zoom API.

## Fix

Import `select` from `d3-selection` and use `.call()`:
```typescript
import { select } from 'd3-selection';
select(this.canvas).call(zoom);
```

## Why This Happens

`d3-selection@3.0.0` is already a transitive dependency of `d3-zoom@3.0.0` (via `d3-drag`), so no additional package install is needed.

## Lesson

Always wrap raw DOM elements with `d3.select()` before passing to d3 behaviors. TypeScript's `as any` cast hides the API mismatch — don't rely on it.
