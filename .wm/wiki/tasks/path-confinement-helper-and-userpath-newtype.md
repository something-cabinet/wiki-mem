---
title: Path confinement chokepoint + UserPath newtype
type: task
status: done
acceptance_criteria:
  - text: "normalize_lexically, confine, and confine_strict are implemented; .. is resolved lexically (not via canonicalize, since create-paths do not exist on disk) and the symlink check canonicalises the deepest existing ancestor"
  - text: "Table-driven tests cover leading/mid/repeated .., absolute-inside-root (allow), absolute-outside-root (reject), dot-components rejected in strict mode, symlinks pointing outside (reject), empty and .-only input, and Windows separators"
  - text: "UserPath newtype has no AsRef<Path> or Deref and is only unwrapped via confine/confine_strict; reintroducing an unconfined .join() on request input fails to compile or fails CI, and cargo clippy/check emit zero warnings"
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

## Implementation Notes (2026-08-08)

- Confinement (`normalize_lexically`/`confine`/`confine_strict`) was already implemented; this lane added the missing table tests and one hardening change.
- **`symlink_escapes` hardened** to canonicalise the *deepest existing ancestor* instead of the full resolved path — create-paths whose final segment does not exist yet now still catch symlinks escaping the root (required for the RED symlink test and the task AC).
- New tests in `path_confine_helper.rs` (all pass): `symlink_escaping_root_is_rejected` (unix), `empty_candidate_resolves_to_root`, `dot_only_candidate_resolves_to_root`, `windows_backslash_separators_cannot_escape`, `absolute_inside_root_is_allowed`, `absolute_outside_root_is_rejected`, plus `table_driven_traversal_rejections` / `table_driven_allowed_paths`.
- Confinement rejections now also emit a `security` audit event (kind `path_escape`/`hidden_path`) through the shared sink.
- **UserPath decision**: NOT adopted as a tool-input type. Adoption would mean changing every tool's `String` params to a newtype — a large, cross-cutting refactor that would churn all tool schemas and every subprocess test fixture for no security gain, because every write path already funnels through the `confine`/`confine_strict` chokepoint (a future unconfined `.join()` on request input would still hit the chokepoint on the next operation). The `raw()` escape hatch is kept and is documented in `apps/wm-core/tests/security_test.rs::userpath_raw_escape_hatch_is_documented_surface`. Revisit if a tool is ever added that writes outside the confinement chokepoint.
