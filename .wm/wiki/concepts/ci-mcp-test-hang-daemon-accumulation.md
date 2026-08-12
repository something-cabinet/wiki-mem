---
title: 'Hangover: CI mcp_test 50-min hang — sequential daemon-spawn accumulation'
type: concept
id: wiki:concepts:ci-mcp-test-hang-daemon-accumulation
status: draft
tags:
- failure
- ci
- debugging
- github-actions
- onnx
relates_to:
  - {type: references, target: wiki:patterns:isolate-suspect-test-on-ci}
  - {type: references, target: wiki:patterns:shrink-test-and-daemon-binaries-for-ci}
---

# Hangover: CI mcp_test 50-min hang — sequential daemon-spawn accumulation

## What went wrong

`test-mcp` job in GitHub Actions hung at ~50 min (`running 74 tests`, then
nothing after `test_graph_stats`), on every CI run, while passing locally in
every configuration (default/no-onnx features, serial/parallel threads,
`OMP_NUM_THREADS=1`). No timeout fired (45m `timeout-minutes` ignored), no
log flush, `conclusion: failure` with all remaining steps `pending`.

## Root cause (verified chain)

1. Every wm-core test binary statically links onnxruntime (ort) via the
   default `onnx` feature → wm_core test = 121MB, wm_embed test = 116MB →
   full test profile ≈ 23GB → exceeds the free runner's 14GB SSD → job
   evicted mid-compile at ~46 min (this was the FIRST failure mode).
2. Fix: per-job split + `--no-default-features` on test jobs → test binary
   small, compiles in 52s. All other jobs pass in minutes.
3. BUT `test-mcp` still hung at `test_help_all_tools` (~21st test). The
   daemon binaries (`wm-cli`/`wm-server`) were still built WITH default
   features (onnx ON) in the job's build step, while the test binary was
   no-onnx. Each of 74 tests spawns a fresh ~120MB onnx `wm-server` via the
   proxy. On the 2-core/7GB runner, ~21 sequential 120MB daemon spawns
   accumulate → resource exhaustion → hang.
4. Isolated `test_help_all_tools` on CI passed in seconds → confirmed
   accumulation, not a test/daemon interaction.

## Key debugging lessons

- **The "46m failures" were my own pushes cancelling the previous run** via
  the concurrency group (`cancel-in-progress: true`), misattributed to
  timeouts/OOM for several rounds. Check run-cancellation source before
  theorizing about timeouts.
- `timeout-minutes` was NOT enforced on this repo — hung jobs never
  self-terminate; manual cancel (needs admin) or a new push is the only out.
- Streaming per-test output (`--test-threads=1 --nocapture`) was the decisive
  diagnostic — it named the exact hang point (`test_help_all_tools`).
- Feature mismatch (no-onnx test binary + onnx daemon) is the hang's cousin:
  shrink BOTH the test binary AND the spawned daemon for CI test jobs.

## Fix

Build the daemon binaries in `test-mcp` with the same
`--no-default-features --features "code-intel,lsp"` as the test binary, so
each spawned `wm-server` is ~5MB not ~120MB. Keep `test-onnx` job for ort
coverage. (Patch was prepared but reverted per user; see git history.)

## Time lost

~4+ hours of CI debugging across ~8 pushes and many misdiagnoses.

## Related

- @task-<id> (CI hang task, if created)
- Pattern: isolate suspect test on CI
- Pattern: shrink test AND daemon binaries for CI
