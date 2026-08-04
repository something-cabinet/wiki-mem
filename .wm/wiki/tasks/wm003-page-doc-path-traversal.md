---
title: WM-003 — Arbitrary md write, overwrite and delete outside project root
type: task
status: todo
---


Severity: High

`resolve_page_path` guards with `!file_path.starts_with(&wiki_dir)`, which does not resolve `..`. Verified live: `wm_page create` with `path: "../../../fakehome/.evilrc"` created a file outside the project root and returned success with id `wiki:..:..:..:fakehome:.evilrc`.

`.md` is force-appended, so impact is arbitrary `*.md` create/overwrite/delete anywhere the user can write. `doc.rs:393` makes deletion reachable the same way.

The existing test at `page/mod.rs:107-131` is vacuous: it asserts the same broken predicate, so it passes on the traversing path and will pass forever.

## Acceptance Criteria

- [ ] RED: `page/mod.rs:107-131` rewritten to assert `Err`, and it fails against pre-fix code
- [ ] The suite is grepped for the same shape — `match { Ok(_) => assert!(weak), Err(_) => {} }` — and every instance fixed
- [ ] `wm_page create` with `path: "../../../x"` returns `Err` and writes nothing outside `.wm/wiki/`
- [ ] `wm_doc create`, `update`, and `delete` reject traversing paths
- [ ] `resolve_simple_page_path` (currently unguarded) is confined
- [ ] `graph_meta_path_is_relative_to_project_root` still passes
- [ ] `cli_page_crud_from_wiki_root_resolves_meta_path` still passes
- [ ] `e2e_pages.rs`, `e2e_workflow.rs`, `mcp_test.rs` still green
- [ ] REFACTOR: touched `std::fs` calls in `doc.rs` converted to `tokio::fs`
- [ ] Rejections emit `tracing::warn!`
- [ ] `cargo clippy --workspace` and `cargo check --workspace` emit zero warnings

## Files

- `apps/wm-core/src/page/helpers/page_path_helper.rs` (:16 broken guard, :35 no guard at all)
- `apps/wm-core/src/mcp/tools/doc.rs` (:233, :293, :341, :385 broken guards; :312, :364 writes; :393 delete)
- `apps/wm-core/src/page/mod.rs` (:107-131 vacuous test)
- `apps/wm-core/src/mcp/tools/page/action.rs` (:21-23 `path` from request input)

## Notes

Highest regression risk in the remediation: page IDs legitimately contain `:` which becomes path separators, so an over-strict guard breaks normal operations. `path_resolution_test.rs` is the canary — it exercises create/get/update/link/unlink/delete with `:`-separated IDs.
