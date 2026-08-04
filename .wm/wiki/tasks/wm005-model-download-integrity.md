---
title: WM-005 — Model download integrity verification disabled
type: task
status: todo
---


Severity: Medium

Both `MODEL_REGISTRY` entries ship `sha256: ""` with a `TODO`, and the empty case warns and proceeds. The hash is computed, printed, then discarded. The downloaded `.onnx` is loaded into the ORT runtime, so an unverified computation graph from a third-party CDN is executed.

## Acceptance Criteria

- [ ] RED: an empty expected hash returns `Err`, failing before the fix
- [ ] RED: a hash mismatch returns `Err` and removes the partial file
- [ ] GREEN: real SHA-256 pinned for `bge-small-en-v1.5` and `all-MiniLM-L6-v2`
- [ ] Verification happens before the file is moved into place
- [ ] `WM_MODEL_SHA` is restricted to development or removed — an env var overriding pinned integrity is itself a weakness
- [ ] Verification logic is unit-testable without a network round trip
- [ ] `cargo clippy --workspace` and `cargo check --workspace` emit zero warnings

## Files

- `packages/wm-embed/src/services/onnx/mod.rs` (:353-367 registry with empty hashes at :359 and :366; :427-436 empty-hash warn-and-continue)

## Notes

`bge-base-en-v1.5` is advertised in `wm_model list` output but absent from `MODEL_REGISTRY` — downloading it fails with `Unknown model`. Worth confirming whether that is intended.
