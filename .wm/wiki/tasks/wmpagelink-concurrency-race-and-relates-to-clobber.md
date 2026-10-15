---
title: "Wiki Tool Reliability: wm_page.link — concurrency race and relates_to clobber"
type: task
id: "wiki:tasks:wmpagelink-concurrency-race-and-relates-to-clobber"
status: in-review
priority: high
tags: [bug, tool-reliability, mcp, graph, frontmatter]
relates_to:
  - {type: references, target: wiki:rules:tool-reliability-bug-tracking}
  - {type: references, target: wiki:rules:durable-write-or-error}
  - {type: references, target: wiki:specs:wmpagelink-merge-and-write-serialization}
assignee: '@me'
implementation_notes: |-
  Both root causes fixed in apps/wm-core. (1) FsPageRepo::write now uses a per-call unique temp name (pid + AtomicU64 sequence), so concurrent writes cannot collide on rename. (2) link_page runs the read-merge-write inside a process-wide per-path lock so concurrent links cannot lose updates; handle_link then re-reads the page and errors if the edge is absent. wm_page.update keeps replace semantics. Tests: link_sequential_keeps_both_edges, link_preserves_update_authored_edge, link_parallel_keeps_all_edges (stable over 5 runs); full wm-core suite passes except the 2 pre-existing ONNX env tests. Pending: commit + rebuild/install the binary (running binary is 0.5.2).

  ## 2026-10-15 — `wm_page.link` still clobbers `relates_to` (regression after e437059)

  Reproduced while linking a task to a spec and three new pages.

  - Input: four sequential `wm_page({"action":"link","id":"<task>","target":"<X>","edge_type":"references"})` calls — targets, in order: `wiki:specs:rust-learning-knowledge-capture`, `wiki:howto:rust-inspect-variables-and-layout`, `wiki:patterns:borrow-errors-order-area-layout`, `wiki:concepts:rust-learning-research-sources`.
  - Output: all four returned `{"status":"linked","type":"references"}`.
  - Counter-evidence: the task file frontmatter contains **exactly one** edge — the last one written (`wiki:concepts:rust-learning-research-sources`), at line 16. The earlier three are gone.
  - Root cause: each `link` call replaces the `relates_to` array instead of merging; the final write wins. Commit `e437059 fix(page): serialize writes and merge link edges` did not hold for the sequential-link case.
  - Consequence: multi-edge linking is only possible if every edge is written in a single call, and the success payload still claims each individual link succeeded. Violates rule `durable-write-or-error`.
  - Workaround: batch all edges into one `wm_page.update` with `relates_to`, or author `@wiki/...` body references (which the graph builder picks up) instead of frontmatter edges.

  Second instance of the same defect class seen today: `wm_task check_ac` also reports success without persisting (see the note above in this file). Both are 'success payload lies about persistence'.
acceptance_criteria:
  - text: "Concurrent wm_page.link calls on the same source page no longer fail with IO_ERROR 'No such file or directory (os error 2)'"
    checked: true
  - text: "wm_page.link appends to relates_to instead of replacing it: an existing entry authored in frontmatter survives a link call"
    checked: true
  - text: "A regression test calls link twice sequentially on one source page and asserts BOTH targets remain in the file's relates_to"
    checked: true
  - text: "A concurrency test fires N parallel link calls on one source page and asserts no IO_ERROR and N distinct edges"
    checked: true
implementation_plan: |-
  ## Root causes (verified in source)

  - Clobber: WmPageAction::Link (apps/wm-core/src/mcp/tools/page/mod.rs:396-402) passes a SINGLE-element relates_to; apply_relates_to (page/services/page_update_builder_service.rs:169-186) removes the whole relates_to block then writes only that list, so existing edges are destroyed.
  - Race: FsPageRepo::write (apps/wm-core/src/page_repo/mod.rs:20-34) writes to a temp name ".{file}.tmp-{pid}" — NOT unique per call. Two concurrent writes in the same process (one MCP server) share the temp path; the first rename consumes it and the second rename fails with ENOENT -> IO_ERROR "No such file or directory (os error 2)".

  ## Implementation Plan

  1. Make FsPageRepo::write temp names unique per call (pid + monotonic AtomicU64 counter) so concurrent writes cannot collide on rename. (AC-1)
  2. Serialize the read-modify-write in the page update path with a per-path lock so concurrent link/unlink cannot lose updates. (AC-1, AC-4)
  3. Merge in the Link handler: read the existing relates_to from the page, append the new (type, target), dedupe, write the merged list; re-read the page and error if the edge is absent. (AC-2)
  4. Add tests: (a) two sequential link calls keep BOTH edges in the frontmatter; (b) N parallel link calls on one source page produce no IO_ERROR and N distinct edges on disk. (AC-3, AC-4)
  5. Verify: cargo test -p wm-core (scoped; no workspace-wide cargo fmt), plus a manual concurrency repro against the rebuilt binary.

  Files: apps/wm-core/src/page_repo/mod.rs, apps/wm-core/src/mcp/tools/page/mod.rs, apps/wm-core/src/page/services/page_update_builder_service.rs, apps/wm-core/tests/mcp_test.rs.

  Risk: step 2 touches the shared page-write path (all page writes) — keep the lock scope minimal; the existing suite covers regressions.

  Assumptions: single-process MCP server (no cross-process locking needed); relates_to replacement remains the documented behaviour of wm_page.update, while link gains merge semantics.
---

schema_version: 1
state: |-
  Observed 2026-10-13 while creating extraction edges for wiki:specs:wmtask-checkac-persistence-fix. Two independent defects in `wm_page.link`, both reproduced.

  ## Defect 1 — concurrency race on the source page file

  N concurrent `wm_page.link` calls with the SAME `id` race on rewriting that page's frontmatter. The losers fail with `IO_ERROR ... No such file or directory (os error 2)`.

  Reproduction (5 concurrent, one source page):
  ```
  wm_page.link(id="wiki:concepts:linkrace-scratch", target="wiki:patterns:verify-writes-by-read-back", edge_type="references")   -> IO_ERROR No such file or directory (os error 2)
  wm_page.link(id="wiki:concepts:linkrace-scratch", target="wiki:rules:durable-write-or-error", edge_type="references")          -> {"status":"linked"}
  wm_page.link(id="wiki:concepts:linkrace-scratch", target="wiki:specs:wmtask-checkac-persistence-fix", edge_type="references")  -> IO_ERROR No such file or directory (os error 2)
  wm_page.link(id="wiki:concepts:linkrace-scratch", target="wiki:core:critical-patterns", edge_type="references")               -> IO_ERROR No such file or directory (os error 2)
  wm_page.link(id="wiki:concepts:linkrace-scratch", target="wiki:decisions:zero-based-ac-index-and-read-back", edge_type="references") -> IO_ERROR No such file or directory (os error 2)
  ```
  Result: 4 of 5 failed (~174 ms). The same 4 calls run sequentially all returned `{"status":"linked"}` — so the failure is concurrency, not the input.

  Full error output (identical for every failure):
  `{"code":"IO_ERROR","error":"No such file or directory (os error 2)"}`

  ## Defect 2 — link REPLACES relates_to instead of appending

  Each `wm_page.link` rewrites the page's whole `relates_to` block from a snapshot that did not include the entries already on disk, so every call clobbers the previous edges — including edges written directly into the frontmatter.

  Counter-evidence (files are the source of truth; read after the calls):
  - `wiki:rules:durable-write-or-error` frontmatter authored two edges (task + spec). After `link(→task)` then `link(→spec)`, the file contains ONLY the spec entry. The task edge is gone.
  - Sequential control on the scratch page: 1 existing edge + 4 sequential `link` calls → the file ends with exactly 1 edge (the last target). All four are reported `{"status":"linked"}`.

  So the tool returns success while dropping data — the same class as `wiki:tasks:wmtask-checkac-and-status-updates-report-success-without-persisting`: a success payload is not proof of a durable, correct write (see `@wiki/rules/durable-write-or-error`).

  ## Impact

  Graph edges are silently lost. Extraction/step-4 flows that create several edges on one page keep only the last, and any edge authored in frontmatter is destroyed by the next `link` call. The graph (and `wm_graph.neighbors`) under-reports relations, and there is no error to signal it.

  ## Workaround

  Do not rely on `wm_page.link` to accumulate edges. Author `relates_to` directly in the page frontmatter (block form, as the other pages do), then `wm index rebuild`:
  ```yaml
  relates_to:
    - {type: references, target: wiki:tasks:<id>}
    - {type: implements, target: wiki:specs:<id>}
  ```
  Do not issue `wm_page.link` concurrently. If you must use it, call it sequentially and re-read the file to confirm prior entries survived.

  ## Reproduction steps

  1. Create a scratch page: `wm_page.create(id="wiki:concepts:linkrace-scratch", path="concepts/linkrace-scratch", title="scratch")`.
  2. Fire 5 `wm_page.link` calls with the same `id` and different `target`s in parallel: 4 fail with `IO_ERROR`.
  3. Run the same 4 calls sequentially: all succeed.
  4. Read the scratch file's `relates_to`: only the last target remains, not 5.
  5. Delete the scratch page and `wm index rebuild`.

  ## Expected behaviour

  1. `link` must serialize (or lock) writes to the source page; concurrent calls either all succeed or return a non-IO error.
  2. `link` must MERGE with the on-disk `relates_to`, never replace it. Round-trip safety per `wiki:specs:typed-decision-doc-format` NFR-2: never whole-block YAML round-trip; use the line-based helpers.
  3. A failed link must not report `{"status":"linked"}`.

  ## Related

  - @wiki/tasks/wmtask-checkac-and-status-updates-report-success-without-persisting — same silent-write-loss class
  - @wiki/rules/durable-write-or-error
  - @wiki/patterns/verify-writes-by-read-back
questions:
  - id: work_kind
    type: choice
    instructions: What kind of work is this task?
    options:
    - feature
    - bugfix
    - refactor
    - docs
    - test
    - chore
    - migration
  - id: priority
    type: choice
    instructions: What priority is this task?
    options:
    - low
    - medium
    - high
    - urgent
  - id: needs_spec
    type: noul
    instructions: This task depends on a spec.
  - id: has_ac
    type: noul
    instructions: This task has at least one acceptance criterion.
answers: {}