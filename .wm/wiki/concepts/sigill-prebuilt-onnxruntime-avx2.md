---
title: 'Failure: SIGILL from prebuilt ONNX Runtime requiring AVX2'
id: wiki:concepts:sigill-prebuilt-onnxruntime-avx2
type: concept
relates_to:
  - {type: references, target: wiki:memory:sigill-root-cause-prebuilt-libonnxruntime-a-requires-avx2}
---

---
title: Failure: SIGILL from prebuilt ONNX Runtime requiring AVX2
type: concept
id: wiki:concepts:sigill-prebuilt-onnxruntime-avx2
tags: [failure, debugging, sigill, onnx, ci]
---

## What went wrong
CI test binaries crashed with SIGILL (illegal instruction, signal 4) on a self-hosted Gitea runner. Multiple attempted fixes (RUSTFLAGS, CFLAGS, cargo clean) all failed.

## Root cause
The `ort` crate (ONNX Runtime bindings) downloads a **prebuilt `libonnxruntime.a`** static library at build time. This binary blob was compiled with AVX2 instructions in **unconditionally-executed code paths** (no CPU feature gating or runtime dispatch). The CI runner's CPU lacked AVX2 support, causing SIGILL on every test binary that linked ONNX.

RUSTFLAGS and CFLAGS had no effect because the library is a pre-compiled binary, not compiled from source during the build.

## Key clues
- SIGILL was **immune to all compiler flags** (`-C target-cpu=generic`, `-march=x86-64`)
- Even `cargo clean` didn't help (the binary was re-downloaded each time)
- Other projects (without `ort`) compiled and tested fine on the same runner
- The crash happened **before any test code ran** — on static initialization of the ORT environment

## Prevention
1. When SIGILL is immune to `target-cpu=generic` + `CFLAGS`, suspect prebuilt/static-linked third-party binaries first.
2. Use `is_x86_feature_detected!` or `objdump` on the binary to check for specific instructions.
3. On GitHub Actions, standard `ubuntu-latest` runners support AVX2 — the issue only affects older or constrained CPUs.
4. For CI without AVX2: exclude the `onnx` feature from test commands.

## Time lost
~3+ hours of debugging, 5+ CI pushes, exploring wrong root causes (simsimd, cc crate CFLAGS, stale cache).

## Related
- @wiki/memory/sigill-root-cause-prebuilt-libonnxruntime-a-requires-avx2