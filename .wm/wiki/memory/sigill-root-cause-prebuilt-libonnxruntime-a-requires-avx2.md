---
title: SIGILL root cause — prebuilt libonnxruntime.a requires AVX2
type: memory
tags: [sigill, ort, onnxruntime, ci, avx2, debugging]
status: active
---

The persistent SIGILL in CI was caused by ort's prebuilt libonnxruntime.a (downloaded at build time), compiled with AVX2 in unconditional init paths. The CI runner's CPU lacked AVX2 support. Our RUSTFLAGS/CFLAGS/cargo clean fixes had no effect because the library is a pre-compiled binary blob, not compiled in CI.

Fix: run tests without `--features onnx` when the runner doesn't have AVX2:
`cargo test -p wm-core --no-default-features --features code-intel,lsp -- -q`

First lesson: when SIGILL is immune to target-cpu=generic + CFLAGS, suspect prebuilt/static-linked third-party binaries first.