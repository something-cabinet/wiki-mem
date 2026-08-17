---
title: Code index goes stale — affected answers from a 10-day-old code.db
type: task
id: "wiki:tasks:code-index-goes-stale--affected-answers-from-a-10-day-old-codedb"
status: cancelled
priority: high
tags: [bug, code-intel, staleness, index, graphify]
acceptance_criteria:
  - text: "Code index refresh is wired at the write path for code files, mirroring graph::handle_file_change for wiki pages (pattern wiki:patterns:refresh-derived-state-at-write-path)"
  - text: "The daemon's notify watcher covers source files, not only .wm/wiki, so external edits invalidate affected code index entries"
  - text: "wm_code.search and the graph code path read via CodeIndexSnapshot::from_db; collect_from_fs remains only as the no-index fallback and is no longer the hot path"
  - text: "A regression test writes a source file, then asserts a code-edge query reflects the change without a manual wm index code run"
  - text: "wm index code reports index age or staleness so a stale code.db is visible rather than silently served"
implementation_notes: Absorbed into wiki:specs:code-edge-resolution Phase 1 (2026-08-14). Cancelled rather than done — no code has changed; the finding and its measured evidence live on in the spec Overview, and execution is tracked by wiki:tasks:code-edge-resolution-01-refresh-the-code-index-at-the-write-path-and-via-the-watcher plus wiki:tasks:code-edge-resolution-02-read-the-index-instead-of-rewalking-and-report-index-age. Task status vocabulary has no superseded value, so cancelled is the closest allowed state.
---

Verified 2026-08-14T17:58 on this repo. .wm/state/code.db was last written 2026-08-04 11:51 (WAL 16:33); 158 source files under apps/ and packages/ have been modified since. apps/wm-core/src/graph/code_edges.rs:73 loads the code-edge graph via CodeIndexSnapshot::from_db, so wm graph affected answers blast-radius questions from a ten-day-old snapshot with no staleness signal. Root cause: nothing refreshes the code index. The wiki graph has both a synchronous write-path refresh (graph::handle_file_change after every page write) and the daemon notify watcher; the code index has neither, so it only advances when a human runs wm index code. Consequence chain: because code.db cannot be trusted, apps/wm-core/src/mcp/tools/code.rs:149 bypasses it and calls collect_from_fs, re-walking and re-parsing all 422 source files on every wm_code.search call. That bypass is a correctness workaround for this staleness bug, not an optimization oversight — fixing freshness is the precondition for removing it. The repo already documents the applicable pattern as wiki:patterns:refresh-derived-state-at-write-path; it was applied to the wiki graph and never to the code index.