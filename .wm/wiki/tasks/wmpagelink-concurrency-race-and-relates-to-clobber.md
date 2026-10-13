---
title: "Wiki Tool Reliability: wm_page.link — concurrency race and relates_to clobber"
type: task
id: "wiki:tasks:wmpagelink-concurrency-race-and-relates-to-clobber"
status: todo
priority: high
tags: [bug, tool-reliability, mcp, graph, frontmatter]
acceptance_criteria:
  - text: "Concurrent wm_page.link calls on the same source page no longer fail with IO_ERROR 'No such file or directory (os error 2)'"
    checked: false
  - text: "wm_page.link appends to relates_to instead of replacing it: an existing entry authored in frontmatter survives a link call"
    checked: false
  - text: "A regression test calls link twice sequentially on one source page and asserts BOTH targets remain in the file's relates_to"
    checked: false
  - text: "A concurrency test fires N parallel link calls on one source page and asserts no IO_ERROR and N distinct edges"
    checked: false
relates_to:
  - {type: references, target: wiki:rules:tool-reliability-bug-tracking}
  - {type: references, target: wiki:rules:durable-write-or-error}
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
