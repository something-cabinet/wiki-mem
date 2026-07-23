---
title: "Tauri event listeners must be registered before firing the IPC command"
type: learning
status: active
tags: [tauri, events, frontend, race-condition, angular]
relates_to:
  - {type: references, target: wiki:tasks:review-event-listener-race-condition}
---

In Tauri apps, when using dynamic `import('@tauri-apps/api/event')` + `listen()`
for progressive events, the listeners must be fully awaited **before** calling the
backend command that emits them.

If `computeLayout()` IPC fires before `listen('graph-coarse', ...)` resolves,
early events (like the coarse positions at tick 30) are lost and the user never
sees progressive rendering.

**Fix:**
```ts
import('@tauri-apps/api/event').then(async ({ listen }) => {
  const unsub1 = await listen('graph-coarse', handler);
  const unsub2 = await listen('graph-settled', handler);
  // ALL listeners registered — now safe to fire IPC
  this.api.computeLayout(...).subscribe(...);
});
```

**Reference:**
- `apps/wm-web/src/app/views/graph/graph-view.component.ts` — `startLayout()`
- @wiki/tasks/review-event-listener-race-condition
