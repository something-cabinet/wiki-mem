---
title: "Fix: detect_project_root symlink edge case"
type: task
status: done
tags: [review, backend, edge-case, robustness]
priority: low
---

# Fix: `detect_project_root()` symlink edge case

## Description

The new `detect_project_root()` in `apps/wm-web/src-tauri/src/lib.rs` walks up the filesystem looking for `.wm`. If any ancestor directory is a symlink, `dir.pop()` after `dir.join(".wm")` may not correctly pop the symlinked name — could cause infinite loops or misdetection.

## Location

`apps/wm-web/src-tauri/src/lib.rs` — `detect_project_root()`

## Acceptance Criteria

- [ ] Add a maximum walk depth (e.g., 20 levels) as a safety limit
- [ ] Consider using `fs::canonicalize()` to resolve symlinks before walking
- [ ] Test in a directory tree with symlinks in ancestor paths
