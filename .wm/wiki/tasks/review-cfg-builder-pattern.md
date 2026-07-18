---
title: "Refactor: cfg-dependent builder pattern in lib.rs"
type: task
status: done
tags: [review, backend, refactor, cfg]
priority: low
---

# Refactor: cfg-dependent builder pattern in `lib.rs`

## Description

In `apps/wm-web/src-tauri/src/lib.rs`, the Tauri builder was changed from `let mut builder` with a `#[cfg(debug_assertions)] { builder = builder.plugin(...); }` block to a shadowed `let builder` pattern:

```rust
let builder = tauri::Builder::default()...
#[cfg(debug_assertions)]
let builder = builder.plugin(tauri_plugin_pilot::init());
```

This works because Rust allows shadowing under `#[cfg]`, but it's fragile — adding another cfg-dependent builder call below will hit a "consumed value" compile error. The original `let mut builder` pattern is more extensible.

## Location

`apps/wm-web/src-tauri/src/lib.rs` — `run()` function

## Acceptance Criteria

- [ ] Revert to `let mut builder` pattern (or keep if preferred with a comment about the trade-off)
- [ ] Document why the chosen pattern was used
