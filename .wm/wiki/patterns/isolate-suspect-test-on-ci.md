---
title: 'Pattern: Isolate a suspect test on CI to separate accumulation vs interaction'
type: pattern
id: wiki:patterns:isolate-suspect-test-on-ci
status: draft
tags:
- pattern
- ci
- debugging
- testing
relates_to:
  - {type: references, target: wiki:concepts:ci-mcp-test-hang-daemon-accumulation}
  - {type: references, target: wiki:patterns:shrink-test-and-daemon-binaries-for-ci}
---

## Problem

A test suite hangs on CI but passes locally in every configuration. You have
a streaming log showing tests complete in order, then silence — but you don't
know whether the hang is *the test itself* (test/daemon interaction) or
*resource accumulation* (N prior tests each leaked a process/memory until the
N+1th spawn can't proceed).

## Solution

Add a **diagnostic job that runs ONLY the suspect test alone**, with a short
`timeout-minutes` (10m) so it resolves fast:

```yaml
diag-suspect:
  runs-on: ubuntu-latest
  timeout-minutes: 10
  steps:
    - uses: actions/checkout@v4
    - uses: actions-rust-lang/setup-rust-toolchain@v1
    - name: Build prerequisites
      run: cargo build -p <daemon-crate>
    - name: Isolated suspect test
      run: cargo test -p <crate> --test <suite> <suspect_test> -- --nocapture
```

Then interpret:

- **Passes alone** → accumulation: fix by reducing per-test resource usage
  (smaller spawned processes, daemon reuse, fewer parallel spawns).
- **Hangs alone** → test/daemon interaction: the `--nocapture` output shows
  exactly where it blocks.

## When to Use

Any CI-only hang where the local run is green and the log ends mid-suite.
Pair with `--test-threads=1 --nocapture` on the full suite first so the
streaming log names the last completed test — the hang is the next one
alphabetically.

## When Not to Use

- Compile-time hangs (no test output at all) — different root cause (disk/memory during build).
- Tests that fail (not hang) — just read the failure.

## Related

- Hangover: CI mcp_test hang — sequential daemon-spawn accumulation
- Pattern: shrink test AND daemon binaries for CI
