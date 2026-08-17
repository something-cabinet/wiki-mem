---
title: code-edge-resolution-01 Refresh the code index at the write path and via the watcher
type: task
id: "wiki:tasks:code-edge-resolution-01-refresh-the-code-index-at-the-write-path-and-via-the-watcher"
status: done
priority: high
tags: [from-spec, spec:code-edge-resolution, p1, code-intel, freshness]
spec: wiki:specs:code-edge-resolution
acceptance_criteria:
  - text: "Write a source file that adds a call, then query code edges without running wm index code — the new edge is present (spec AC-1.1)"
  - text: "Delete a source file — its symbols and edges disappear from query results without a manual rebuild (spec AC-1.2)"
  - text: "A single-file edit re-extracts only that file, with no full re-index (spec NFR-1.1)"
  - text: "The watcher path is exercised by a test that writes to disk and polls with a deadline, not by calling the handler directly"
relates_to:
  - {type: implements, target: wiki:specs:code-edge-resolution}
time_started: 2026-08-14T11:26:37.921972+00:00
implementation_plan: |-
  Two refresh mechanisms feed one incremental path. Neither is a new layer — both call the existing content-hash incremental rebuild.

  Why two — verified 2026-08-14. apps/wm-cli/src/main.rs:570 and apps/wm-server/src/main.rs:32 both construct MainEngine::with_root, so both get a watcher. But a one-shot CLI invocation constructs, runs and exits, so the watcher cannot help it. Spec AC-1.1 requires that a query reflects a fresh edit without a manual wm index code, which on the CLI path is only achievable with a read-time staleness probe.

  Step 1 — expose the predicates (packages/wm-code-intel)
  Make is_skipped_dir public (currently pub(crate) at ingest_service.rs:15) and add a small is_code_file predicate keyed on SupportedLanguage::from_ext, so wm-core filters watcher events with exactly the same rules the walker uses. No duplicated skip list.

  Step 2 — bound the watch scope (apps/wm-core/src/engine/main_engine_factory.rs)
  Do NOT watch project_root recursively — target/ and node_modules/ would be registered at OS level and flood the channel during every cargo build. Instead enumerate top-level entries of project_root, skip hidden and is_skipped_dir matches, and register a watch per surviving directory on the existing debouncer. The existing wiki watch stays exactly as it is.
  Deliberately NOT reusing config.source_dirs — it defaults to docs/ and specs/ and is the wm_source security boundary (see WM-004), a different concept. Overloading it would break that boundary. No new config field either; the code roots derive from the skip list plus supported extensions.

  Step 3 — route events by kind (same watcher thread)
  Keep the current .md branch untouched so graph::handle_file_change and handle_file_delete behave identically. Add a branch for code files that triggers the code-index refresh. Preserve the existing early-continue filters.

  Step 4 — refresh incrementally
  Call the existing rebuild_code_index. It is already content-hash incremental, so only changed files are re-parsed, satisfying NFR-1.1. The 500ms debouncer already coalesces edit bursts.

  Step 5 — read-time staleness probe for one-shot invocations
  Use the existing scan_file_metadata(project_root) to compare source state against the stored index timestamp, and trigger the same incremental rebuild when stale. This is what makes AC-1.1 pass on the CLI path and gives FR-1.4 its age value for free.

  Step 6 — do not regress the watcher-thread spawn fix (NFR-1.3)
  engine_state_mediator.rs:158 notify_file_changed already uses Handle::try_current with an rt.block_on fallback because tokio::spawn from a std thread panics with no reactor running and dies silently. Add no bare tokio::spawn in the watcher thread; if async work is needed, reuse that pattern.

  Step 7 — report index age (FR-1.4)
  Surface index age in wm index code output and wm_index_status, so a stale index is visible rather than silently served.

  Step 8 — tests, RED first per wiki:rules:tdd-red-green-refactor
  - Watcher-level - write a .rs file into a temp project, poll with a deadline until the new symbol or edge appears (AC-1.1, AC-1.5). Test the real thread, not the handler, per wiki:core:critical-patterns.
  - Deletion - remove a source file, poll until its symbols and edges are gone (AC-1.2).
  - Scope guard - churn under target/ or node_modules/ triggers no re-index.
  - Parse accounting - with an index present, only changed files are parsed (AC-1.4) by counting parse invocations, never wall clock.
  - Age reporting - a deliberately old index reports as stale (AC-1.3).
  - Regression - the wiki .md watcher path still refreshes the graph.

  Step 9 — verify
  cargo check --workspace and cargo clippy --workspace with zero warnings per wiki:rules:no-warnings, then the wm-code-intel and wm-core suites. Confirm the watcher code still compiles with the code-intel feature disabled.

  Out of scope - reading from_db on the hot path and removing the collect_from_fs bypass, which is task 02 and depends on this landing first.
implementation_notes: |-
  ## Implementation complete 2026-08-17 — awaiting review

  Status set through wm_page because wm_task update dropped it (see wiki:tasks:wmtask-checkac-and-status-updates-report-success-without-persisting). Acceptance criteria are NOT ticked in frontmatter because check_ac does not persist; evidence for each is recorded here instead.

  ### What landed

  - packages/wm-code-intel ingest_service.rs — is_skipped_dir made pub so wm-core filters watcher events with the same rules the walker uses. No duplicated skip list.
  - apps/wm-core/src/engine/code_index_refresh_service.rs (new) — refresh_code_index for the watcher, refresh_if_stale for one-shot invocations. Both delegate to the existing content-hash incremental rebuild. Neither creates an index that wm index code has not built, so code intelligence stays opt-in.
  - apps/wm-core/src/engine/main_engine_factory.rs — watcher now registers top-level source trees alongside .wm/wiki and routes events by kind. New predicates is_wiki_page, is_code_source, code_watch_roots plus three unit tests.
  - apps/wm-core/src/graph/code_edges.rs — load_code_graph runs the staleness probe first, which is what makes the fix reachable from the CLI.
  - apps/wm-core/tests/code_index_watcher_test.rs (new) — 7 tests.

  ### Evidence per acceptance criterion

  - AC-1 (edit visible without wm index code) — MET. watcher_indexes_new_source_file covers the long-lived path; code_graph_read_refreshes_a_stale_index covers the one-shot path.
  - AC-2 (deletion propagates) — MET. watcher_removes_deleted_source_file.
  - AC-3 (single-file edit re-extracts only that file) — MET by delegation to rebuild_code_index content hashing, covered by the pre-existing incremental tests. Caveat below on the metadata walk.
  - AC-4 (watcher exercised through the real thread) — MET. Three tests write to disk and poll with a 20s deadline; none calls the handler directly.

  ### Self-review findings, both fixed before this note

  - P1 dead feature: refresh_if_stale was initially called only by tests, so AC-1 passed in the suite while being unreachable in the shipped binary. Fixed by calling it from load_code_graph. index_lag_seconds had no caller at all and was deleted rather than left as decoration; task 02 adds it when it wires FR-1.4 reporting.
  - P1 quadratic refresh: the watcher called the refresh once per event instead of once per debounced batch, so a 50-file change ran 50 rebuilds. Fixed with a per-batch flag.
  - Regression caught by the existing suite: adding starts_with(wiki_dir) to the wiki branch broke both live watcher tests on macOS, where events arrive canonicalised as /private/var while wiki_dir is /var. Fixed by accepting either prefix and covered by a new unit assertion, plus a case proving markdown outside the wiki cannot reach the wiki graph handler.

  ### Verification

  cargo check --workspace zero warnings. cargo clippy --workspace --all-targets has 9 warnings, none in any file this task touched — all pre-existing in graph/mod.rs, search/query.rs, tests/helpers/ and e2e_http.rs, including 5 stale expect(dead_code) expectations. rustfmt clean on the touched files only, never workspace-wide. Suites green: code_index_watcher_test 7, file_watcher_test 7, graph_code_edges 5, e2e_code_intel 7, mcp_test 54, cli_test 17, wm_cli_web_test 9, path_resolution 3, security 18, e2e_graph 3, e2e_task 2, wm-core lib 160, wm-code-intel 52. Build verified with code-intel disabled.

  ### Gaps, stated plainly

  - Spec AC-1.4 (assert parse invocations, not wall clock) is NOT covered. The instrumentation to count parses does not exist and inventing a timing proxy would be a test that cannot fail honestly. Task 02 owns it.
  - Spec NFR-1.2 says no read path triggers a full repository walk once an index exists. The staleness probe does a stat-only walk of every source file on each load_code_graph call. It reads no contents, but it is a walk. Deliberate tradeoff: the alternative is trusting a possibly stale index, which is the bug being fixed. Task 02 should decide whether to cache the probe per process or key it on directory mtimes.
  - The independent review gate could not run: two subagent dispatches failed with the same upstream stream DecryptError. What is recorded above is a self-review, which is weaker evidence than an external reviewer. Per wiki:core:critical-patterns a dead gate is not a pass, so this task stays in-review rather than done.
---

Phase 1 of wiki:specs:code-edge-resolution. Implements FR-1.1, FR-1.2, NFR-1.1.

Code index entries must refresh when a source file changes, mirroring graph::handle_file_change / handle_file_delete for wiki pages (pattern wiki:patterns:refresh-derived-state-at-write-path). The daemon notify watcher in MainEngine::with_root currently covers .wm/wiki only and must cover source files so external edits invalidate the affected entries.

Evidence for why this is first: .wm/state/code.db was last written 2026-08-04 while 158 source files changed since, and apps/wm-core/src/graph/code_edges.rs loads via from_db — so blast radius answers come from a stale snapshot.

Known hazard recorded in wiki:core:critical-patterns — tokio::spawn called from the watcher's std thread panics with no reactor running, and it fails silently after the graph update has already applied. Use Handle::try_current then fall back to a one-shot current-thread runtime. Test the real watcher thread, not the handler.

Files: packages/wm-code-intel/src/services/ingest_service.rs, apps/wm-core/src/engine (write paths), MainEngine::with_root watcher setup.