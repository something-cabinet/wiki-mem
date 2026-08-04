---
title: WM-001 — Arbitrary recursive directory deletion via wm_model remove
type: task
status: todo
---


Severity: Critical

`wm_model` `remove` joins an unvalidated `name` onto the models directory and calls `remove_dir_all`, giving arbitrary recursive directory deletion. Verified live: `{"action":"remove","name":"../../../precious"}` returned `{"status":"removed"}` and the target directory was gone.

Fix is an allowlist, not path confinement — the valid set is finite and already declared in `MODEL_REGISTRY`, and `download_model` already validates against it.

## Acceptance Criteria

- [ ] RED: a test asserting `remove` with `name = "../../../victim"` returns `Err` and the target survives, failing before the fix
- [ ] GREEN: `name` is validated against `MODEL_REGISTRY` before `remove_dir_all`
- [ ] `{"action":"remove","name":"bge-small-en-v1.5"}` still removes that model directory
- [ ] An unknown model name returns a clean not-found error, not silent success
- [ ] Registry names are exported rather than duplicated (no-magic-values)
- [ ] REFACTOR: touched `std::fs` calls in `model.rs` converted to `tokio::fs`
- [ ] A rejection emits `tracing::warn!` with the attempted name
- [ ] `cargo clippy --workspace` and `cargo check --workspace` emit zero warnings

## Files

- `apps/wm-core/src/mcp/tools/model.rs` (:104-118; join at :110, `remove_dir_all` at :113)
- `packages/wm-embed/src/services/onnx/mod.rs` (`MODEL_REGISTRY` at :353; allowlist use at :377)

## Notes

`model.rs` is named in the rust-anti-patterns rule section 4 for blocking I/O in async context.
