---
title: Pattern — Lexical path normalization for confinement (Path starts_with pitfall)
type: pattern
id: wiki:patterns:lexical-path-confinement
status: draft
---

## Problem

`Path::starts_with` in Rust is component-wise and does NOT resolve `..` segments. This means `.wm/wiki/../../etc/passwd.md.starts_with(".wm/wiki")` returns `true`. Every path-confinement guard that relies on lexical `starts_with` without prior normalization is bypassable.

`canonicalize()` alone is also insufficient because create-paths do not yet exist on disk.

## Solution

Normalize `..` segments lexically before comparison:

```rust
fn normalize_lexically(path: &Path) -> PathBuf {
    let mut out: Vec<Component> = Vec::new();
    for component in path.components() {
        if matches!(component, Component::CurDir) { continue; }
        if matches!(component, Component::ParentDir)
            && matches!(out.last(), Some(Component::Normal(_)))
        { out.pop(); continue; }
        out.push(component);
    }
    out.iter().collect()
}
```

Then `normalized.starts_with(root)` is correct. For symlink escape, canonicalize the deepest existing ancestor and re-check.

For dotfile exclusion (when secrets like `.git/config` sit inside the root), add a second check rejecting components starting with `.` — `confine_strict`.

## When to Use

Any time user/agent-supplied input reaches a filesystem operation — `fs::read`, `fs::write`, `fs::remove_dir_all`, or any path that becomes a `join()` argument.

## When Not to Use

Internal paths derived from your own data (graph node paths, config-derived paths) that never receive external input.

## Related

- wiki:specs:security-remediation
- apps/wm-core/src/shared/helpers/path_confine_helper.rs
