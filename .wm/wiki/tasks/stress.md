---
id: wiki:tasks:stress
title: Stress and Scale Tests
type: task
status: done
priority: low
tags: [testing, stress, performance]
acceptance_criteria:
  - text: "1000-page graph rebuild completes in under 5s and search across 10K documents returns results in under 500ms"
  - text: "10 concurrent daemon connections (MCP channel) run without crashes or data corruption"
  - text: "500 rapid version updates keep compacted file size under 100KB"
---

## Overview

Add stress/scale tests for the WM engine to ensure it handles larger workloads.

## D1 re-spec (oracle)

Since `wm mcp` is now a stdio→HTTP proxy and wm-server is the single daemon,
"concurrent MCP connections" is re-specified as **concurrent connections to the
daemon's privileged MCP channel** (`POST /api/mcp/tools/call`). The stress suite
spawns a real daemon via `tests/helpers/http_daemon.rs` and drives it over HTTP.

## Status: DONE — `apps/wm-core/tests/stress_test.rs`

All four ACs implemented in `stress_test.rs`:

- [x] **AC-1 / TC-14.1** `test_1000_page_graph_rebuild` (`#[ignore]`) — 1000
      pages via CLI, rebuild <5s. Verified passing.
- [x] **AC-2 / TC-14.2** `test_10k_doc_search_benchmark` (`#[ignore]`) — 10K
      docs written to disk, daemon boot, warm-up search, then timed search
      asserts results non-empty and **<500ms**. Verified passing (~8s test time
      incl. fixture build).
- [x] **AC-3 / TC-14.3** `test_concurrent_daemon_connections` — **runs by
      default**: 10 parallel threads hit `/api/mcp/tools/call` (5 create + 5
      list) against one daemon; asserts every call succeeds, all 5 written
      pages survive (no lost writes), and the daemon still serves health after
      the burst. Verified passing in ~0.5s.
- [x] **AC-4 / TC-14.4** `test_version_compaction` (`#[ignore]`) — 500 rapid
      version updates keep `.wm/versions` total <100KB. Verified passing.
- [x] **AC-5** All stress tests pass (`cargo test -p wm-core --test stress_test`
      for the default concurrent test; `-- --ignored` for the heavy benchmarks).

## Runner notes
- Default (CI): `cargo test -p wm-core --test stress_test` — runs only the
  concurrent daemon test.
- Heavy benchmarks: `cargo test -p wm-core --test stress_test -- --ignored`
  (also added as a `stress` release-test note in the file header).
- CI job `e2e-stress` in `.github/workflows/ci.yml` runs the default stress
  suite against a spawned wm-server.
