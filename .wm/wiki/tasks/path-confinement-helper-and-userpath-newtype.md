---
title: Path confinement chokepoint + UserPath newtype
type: task
status: todo
---

Severity: Critical

Foundation for WM-002, WM-003, WM-004. A single confinement chokepoint, enforced by the type system so it cannot be bypassed by a future call site.

Root cause it addresses: `Path::starts_with` is component-wise and does not resolve `..`, so `.wm/wiki/../../etc/passwd.md`.starts_with(`.wm/wiki`) is `true`. Only the absolute-path case is currently caught.

## Acceptance Criteria

- [ ] RED: table-driven tests covering leading/mid/repeated `..`, absolute inside root (allow), absolute outside root (reject), dot-components rejected in strict mode, symlink inside root pointing outside (reject), non-existent create-path inside root (allow), empty and `.`-only input, Windows separators
- [ ] GREEN: `normalize_lexically`, `confine`, `confine_strict` implemented
- [ ] `..` resolved lexically, not via `canonicalize` — create-paths do not exist on disk yet
- [ ] Symlink check canonicalises the deepest existing ancestor
- [ ] Validates WITHOUT absolutising: callers storing relative paths keep storing relative paths
- [ ] `UserPath` newtype has no `AsRef<Path>` and no `Deref`; only `confine`/`confine_strict` unwrap it
- [ ] Reintroducing an unconfined `.join()` on request input fails to compile or fails CI
- [ ] Guard clauses only, no `else`; error strings are named consts; no explanatory comments
- [ ] No bare `unwrap`/`expect` — `?` or `ok_or_else`
- [ ] Barrel re-exports present in `shared/helpers/mod.rs` and `shared/models/mod.rs`
- [ ] `cargo clippy --workspace` and `cargo check --workspace` emit zero warnings

## Files

- `apps/wm-core/src/shared/helpers/path_confine_helper.rs` (new)
- `apps/wm-core/src/shared/models/user_path_model.rs` (new)
- `apps/wm-core/src/shared/mod.rs` (currently only `pub mod traits;`)
- `apps/wm-core/src/mcp/tools/code.rs` (:660-685 is the existing correct reference implementation)

## Notes

FR-3 (no absolutising) exists because `path_resolution_test.rs:38-46` asserts `meta.path` is relative and starts with `.wm/wiki/`. Returning a canonical absolute path breaks that test.

Changing tool input types to `UserPath` will silently rot subprocess tests that call tools by name string — grep tool names across `apps/wm-core/tests/` and update fixtures in the same change.
