---
title: "Fix: race condition in graph layout event listeners"
type: task
status: done
tags: [review, frontend, graph, race-condition, events]
priority: high
---

# Fix: race condition in graph layout event listeners

## Description

In `apps/wm-web/src/app/views/graph/graph-view.component.ts`, the `startLayout()` method fires `this.api.computeLayout(...)` **synchronously** after starting an async `import('@tauri-apps/api/event')` + `listen()` chain. The dynamic import and listener registration are async — if the Tauri backend processes the command before listeners are registered, progressive `graph-coarse`/`graph-refine` events (emitted at tick 30) could be missed.

## Location

`apps/wm-web/src/app/views/graph/graph-view.component.ts` — `startLayout()` method

## Acceptance Criteria

- [ ] Move `computeLayout()` call inside the `import().then(...)` chain, after all listeners are registered
- [ ] OR use `await` pattern: `const { listen } = await import(...)` then `await Promise.all([listen(...), ...])` before firing `computeLayout()`
- [ ] Verify `graph-settled` event is still received (the final positions)
- [ ] Test with cold dynamic import cache (first load)
