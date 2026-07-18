---
title: "Pattern: Workspace Dependency Unification"
type: pattern
tags: [pattern, cargo, build, workspace, dependencies, target-size]
---

## Problem

Rust workspace `target/` directories balloon to tens of gigabytes. Individual crate `Cargo.toml` files declare inline dependency versions instead of referencing `[workspace.dependencies]`, causing Cargo to compile the same dependency multiple times with different feature sets. Each variant produces separate `.lib`, `.rlib`, `.pdb`, and `.rmeta` files that multiply across the workspace.

A 16-crate workspace ballooned to **62.5 GB** — 304 `.lib` files (18.6 GB), 2,061 `.rlib` files (14.3 GB), 308 `.pdb` files (6.4 GB).

## Solution

1. Declare **every shared dependency** in the root `Cargo.toml` under `[workspace.dependencies]` with a single version and feature set.
2. In each crate's `[dependencies]`, reference workspace deps as `{ workspace = true }` instead of repeating version strings.
3. Add crate-specific features inline: `dep = { workspace = true, features = ["crate-specific"] }` only when a crate needs extras beyond the workspace baseline.
4. For deps shared across 2+ crates that aren't yet in `[workspace.dependencies]`, add them to the workspace root first.

### Before

```toml
# apps/wm-core/Cargo.toml
[dependencies]
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
sha2 = "0.10"
```

### After

```toml
# Cargo.toml (root)
[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
sha2 = "0.10"

# apps/wm-core/Cargo.toml
[dependencies]
serde = { workspace = true }
tokio = { workspace = true }
sha2 = { workspace = true }
```

## When to Use

- Any Rust workspace with **3+ member crates**
- Any crate in a workspace that shares dependencies with sibling crates
- When `target/` exceeds 5 GB and keeps growing
- When adding a new dependency to any crate in the workspace

## When Not to Use

- Single-crate projects (no workspace)
- Dependencies used by exactly one crate and unlikely to be shared (though adding to workspace anyway is still fine for centralized version management)

## Impact

12-crate workspace (wm-core, wm-cli, wm-tauri + 9 packages) after unification:

| Metric | Before | After |
|---|---|---|
| **target/ size** | **62.5 GB** (bloated) | **1.62 GB** (clean build) |
| `.lib` files | 304 / 18.6 GB | 43 / 0.1 MB |
| `.rlib` files | 2,061 / 14.3 GB | 211 / 338 MB |
| `.pdb` files | 308 / 6.4 GB | 43 / 131 MB |
| `.rmeta` files | 3,584 / 3.9 GB | 585 / 454 MB |

Clean build time: **1m 02s** (from scratch).

## Related

- @wiki/tasks/workspace-dep-unification (if a task exists)
