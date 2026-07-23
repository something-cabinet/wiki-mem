---
title: "Tauri async commands: yield simulation loops to avoid runtime blocking"
type: memory
status: active
tags: [tauri, async, fjadra, force-layout, performance]
relates_to:
  - {type: references, target: wiki:tasks:review-blocking-async-fjadra-layout}
  - {type: references, target: wiki:specs:stress-scale-tests}
---

When using `#[tauri::command] pub async fn` with a synchronous simulation loop
(e.g., 300 ticks of force layout), the loop blocks the entire Tauri async runtime
for its duration.

**Fix:** Add `tokio::task::yield_now().await` every N iterations (e.g., every 10 ticks)
to yield control back to the runtime. Alternatively, spawn the simulation on
`tokio::task::spawn_blocking()` and emit progress events from there.

**Reference:**
- `apps/wm-web/src-tauri/src/commands.rs` — `compute_layout` function
- @wiki/tasks/review-blocking-async-fjadra-layout
