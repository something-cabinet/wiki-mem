---
title: Verify targeted during iteration, in full only at the end
type: memory
tags: [process, verification, rust]
status: active
---

Run `cargo check -p <crate>` plus only the tests that exercise the change, by name filter, while iterating. Run the full crate test suite and clippy once, after all items are done. Do not run workspace-wide suites between edits, and skip environment-dependent targets (tests/semantic_test fails without an ONNX model and says nothing about the change). Owner instruction, 2026-10-16, after lanes were spending minutes per iteration on full runs.