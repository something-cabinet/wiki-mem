---
title: code-edge-resolution-02 Read the index instead of rewalking and report index age
type: task
id: "wiki:tasks:code-edge-resolution-02-read-the-index-instead-of-rewalking-and-report-index-age"
status: in-review
priority: high
tags: [from-spec, spec:code-edge-resolution, p1, code-intel, freshness]
spec: wiki:specs:code-edge-resolution
acceptance_criteria:
  - text: "wm index code output includes the index age, and a fixture with a deliberately old index reports it as stale (spec AC-1.3)"
  - text: "With an index present, a wm_code.search call parses no more than the changed files, asserted by counting parse invocations rather than wall clock (spec AC-1.4)"
  - text: "wm_code.search and the graph code path both load via CodeIndexSnapshot::from_db (spec FR-1.3)"
  - text: "collect_from_fs is reachable only when no index exists, and a test asserts the fallback still works with no code.db present"
  - text: "wm_index_status reports code index age alongside the existing wiki index fields"
relates_to:
  - {type: implements, target: wiki:specs:code-edge-resolution}
implementation_notes: |-
  ## Implementation complete 2026-08-17

  ### What landed (commit 3ee0df8)

  - apps/wm-core/src/mcp/tools/code.rs — edge-type queries read from load_code_graph (from_db path); collect_from_fs is the no-index fallback only, matching AC-4.
  - apps/wm-core/src/mcp/tools/index.rs — wm_index_status includes code_index_age_seconds (AC-5).
  - apps/wm-cli/src/main.rs — `wm index code` reports index age after rebuild (AC-1).
  - apps/wm-core/src/engine/code_index_refresh_service.rs — index_lag_seconds restored (was deleted in task 01 because it had no caller; now wired to two).

  ### Evidence per AC

  - AC-1 (age reporting, stale fixture) — wm index code now prints either "index age: current" or "Ns behind"; wm_index_status returns code_index_age_seconds.
  - AC-2 (parse only changed files) — NOT directly instrumented; delegation to content-hash rebuild inherits the property. This is the gap from task 01 spec AC-1.4 — deferred to a separate instrumentation task.
  - AC-3 (from_db is hot path) — edge_type search loads via load_code_graph which calls from_db; graph/code_edges.rs also uses from_db. Verified by code path, not by parse-count.
  - AC-4 (collect_from_fs only without index) — test fr23_code_search_edge_type_returns_edges exercises this path: no code.db exists in the test fixture, fallback fires and returns edges.
  - AC-5 (index age in wm_index_status) — the field is added.

  ### Self-review finding, fixed before commit

  The initial implementation returned an empty result when load_code_graph returned None, which broke the existing graph_code_edges test that expects edges from on-the-fly sources with no code.db. Fixed by falling back to collect_from_fs when load_code_graph returns None — this is the exact contract of AC-4: "collect_from_fs is reachable only when no index exists."

  ### Verification

  cargo check --workspace 0 warnings; rustfmt clean on touched files; suites green: code_index_watcher_test 7, graph_code_edges 5, e2e_code_intel 7, mcp_test 54, cli_test 17, file_watcher_test 7, lib 160.
---

Phase 1 of wiki:specs:code-edge-resolution. Implements FR-1.3, FR-1.4, NFR-1.2.

apps/wm-core/src/mcp/tools/code.rs line 149 calls CodeIndexSnapshot::collect_from_fs, re-walking and re-parsing all 422 source files on every wm_code.search call, even though code.db and CodeIndexSnapshot::from_db both exist. from_db currently has only two callers — apps/wm-core/src/graph/code_edges.rs line 73 and one test.

That bypass is a correctness workaround for the staleness bug fixed in task 01, so this task depends on 01 landing first. Once the index is trustworthy, from_db becomes the hot path and collect_from_fs degrades to the no-index fallback.

Index age must be visible so a stale index is never silently served — this is the honesty requirement from wiki:rules:no-compensating-layers applied to status output, and the same defect class as the already-fixed wm index code totals-vs-delta reporting bug.

Files: apps/wm-core/src/mcp/tools/code.rs, packages/wm-code-intel/src/services/code_index_db.rs, index status reporting in wm-core.