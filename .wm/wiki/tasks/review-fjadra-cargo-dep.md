---
title: "Fix: verify fjadra dep in Cargo.toml"
type: task
status: done
spec: specs/webgl-graph-rendering
tags: [review, backend, fjadra, build]
priority: critical
---

# Fix: verify `fjadra` dependency in Cargo.toml

## Description

The `compute_layout` command in `commands.rs` uses `fjadra::force::*` types, but the diff does not show `fjadra` being added as a dependency in `apps/wm-web/src-tauri/Cargo.toml` or the workspace `Cargo.toml`. If missing, the build will fail.

## Location

- `apps/wm-web/src-tauri/Cargo.toml`
- `Cargo.toml` (workspace root)

## Acceptance Criteria

- [ ] Confirm `fjadra` is added as a dependency (workspace dep or direct)
- [ ] Project compiles cleanly
- [ ] Run `cargo check -p wm-web` to verify
