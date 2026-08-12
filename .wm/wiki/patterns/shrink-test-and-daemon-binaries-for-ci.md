---
title: 'Pattern: Shrink test AND daemon binaries for CI'
type: pattern
id: wiki:patterns:shrink-test-and-daemon-binaries-for-ci
status: draft
tags:
- pattern
- ci
- github-actions
- onnx
- binary-size
relates_to:
  - {type: references, target: wiki:concepts:ci-mcp-test-hang-daemon-accumulation}
  - {type: references, target: wiki:patterns:isolate-suspect-test-on-ci}
---

## Problem

Large test binaries blow up CI. Every wm-core test binary statically links
onnxruntime (ort) via a default feature → wm_core test = 121MB, wm_embed test
= 116MB → the full test profile is ~23GB of target/, exceeding the free
GitHub runner's 14GB SSD → jobs get evicted mid-compile (~46 min, no log
flush, `conclusion: failure`). Even after shrinking the *test binary*, a
hanging suite can remain if the *spawned daemon* is still built with the
heavy feature.

## Solution

- **Feature-gate the heavy dep at the source**: make the ort-bearing crate's
  feature optional and non-default. For wm-core: `onnx = ["wm-embed/onnx"]`
  is in `default` — CI test jobs run `--no-default-features
  --features "code-intel,lsp"` so ort never links into test binaries.
- **Shrink BOTH the test binary AND the daemon it spawns**: if tests spawn a
  `wm-server`/`wm-cli` binary via the proxy, that daemon must be built with
  the SAME feature set. A no-onnx test binary spawning a 120MB onnx daemon
  per test = sequential accumulation on small runners (2-core/7GB) → hang at
  ~20th spawn. Build the daemon with the same `--no-default-features`.
- **Dedicated heavy-feature job**: keep ort/onnx coverage in a separate
  `test-onnx` job (embed lib tests + narrow mcp model/index tests with
  `--features onnx`) instead of the default test matrix.
- **Product build unchanged**: `cargo build`/release keeps the heavy feature
  as default — only CI test jobs opt out.

## When to Use

- CI jobs die at a consistent wall-clock (~46 min) with no log flush → suspect
  disk (14GB runner) from oversized test profiles, not memory/timeout.
- Test suite passes locally in every config but hangs on CI at the ~20th
  sequential daemon spawn → suspect spawned-daemon size, not the test.

## When Not to Use

- CI failures with clean logs/errors — read the failure, don't guess.
- Local runs already reproduce the problem.

## Related

- Hangover: CI mcp_test hang — sequential daemon-spawn accumulation
- Pattern: isolate suspect test on CI
