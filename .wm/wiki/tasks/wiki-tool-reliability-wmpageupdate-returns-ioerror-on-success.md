---
title: 'Wiki Tool Reliability: wm_page.update returns IO_ERROR on a successful write'
type: task
id: "wiki:tasks:wiki-tool-reliability-wmpageupdate-returns-ioerror-on-success"
status: todo
tags: [bug, tool-reliability, mcp, update, honesty]
acceptance_criteria:
  - text: wm_page.update returns success (or a documented warning) when the write lands; no IO_ERROR on a successful durable write
  - text: Two concurrent wm_page.update calls on the same page either serialize or both fail loudly; neither silently discards a write
  - text: 'Regression test: a large multi-line state update''s response matches the on-disk outcome'
  - text: 'Concurrency test: no lost update when two writes to the same page are issued in one batch'
priority: high
---

schema_version: 1
state: |-
  ## Bug description

  `wm_page.update` returned an **`IO_ERROR` on a write that fully succeeded**. The reported error was `{"code":"IO_ERROR","error":"No such file or directory (os error 2)"}`, but the target file was updated durably and is structurally valid.

  ## Tool name

  `wm_page` (action `update`)

  ## Full input parameters

  ```json
  {
    "action": "update",
    "id": "wiki:specs:remove-global-memory-layer",
    "content": "<full record envelope: 91 state lines + appended amendment, ~13 KB>"
  }
  ```

  The update was issued alongside two `wm_page` `link` calls in one batch; both links returned success. The update returned the IO_ERROR shown above.

  ## Full error output

  ```
  Error: {"code":"IO_ERROR","error":"No such file or directory (os error 2)"}
  ```

  ## Counter-evidence — the write landed

  - `.wm/wiki/specs/remove-global-memory-layer.md` grew from **11,566 B / 131 lines** to **13,135 B / 142 lines**.
  - The appended text is present: `grep -c 'Amendment (2026-10-15)'` returns **1**.
  - Envelope structure is intact and correctly ordered: `schema_version: 1` at line 9, `state: |-` at line 10, the state block spanning lines 11–112, `questions:` at line 113 (column 0), `answers: {}` at line 142 (column 0).
  - The new `AC-10` line is present at line 112.
  - `wm_page.get` on the same id resolves, reports `status: draft`, and returns **no `record_format_hint` warning** — meaning `parse_record` accepted the body, so the YAML is valid.

  ## Why this matters

  This is the **inverse** of the failure class recorded in `@wiki/rules/durable-write-or-error` and in `@wiki/patterns/verify-writes-by-read-back`: instead of a success payload for a write that did not land, this is an **error payload for a write that did land**. Both are dishonest tool output, and this direction is arguably worse for agents — an agent that trusts the error will either retry (risking duplicate content) or report failure for work that is complete.

  ## Reproduction steps

  1. Read a record-bearing page large enough to have a multi-line `state` block (this case: ~91 state lines pre-append).
  2. Extract the state body, append a section, re-indent, rebuild the envelope, and issue a single `wm_page.update` with that content.
  3. Observe the `IO_ERROR` in the tool response.
  4. Read the file from disk and confirm the content **did** change, and that `wm_page.get` parses it without warning.

  ## Suspected cause (not verified)

  The error names a filesystem path that does not exist, which suggests a **post-write bookkeeping step** — a version record, an index write, or a temporary file — rather than the page write itself. Note the repo has an open task for another path-resolution flaw in this area (`wiki:tasks:fix-update-page-anchored-path` — "wire anchored path resolution into update_page_with_repo"). Investigators should check whether that same resolution is used by the update path's side-effects. This hypothesis is unverified.

  ## Workaround

  Read the file back from disk after any `wm_page.update` and treat the on-disk result as authoritative, not the tool response — which is the behaviour `@wiki/patterns/verify-writes-by-read-back` already prescribes.

  ## 2026-10-15 addendum — the error is concurrency-related, and a same-page race silently loses updates

  Reproduced twice. In both cases the `IO_ERROR` accompanied **two concurrent writes to the same page** issued in a single batch. That correlation is the important discovery, and the lost update is worse than the error.

  **Instance 1 — error false, write landed.** `wiki:specs:remove-global-memory-layer`: a content update (append amendment) was issued concurrently with two `wm_page.link` calls. The update returned `IO_ERROR`; the content **did** land, the envelope stayed valid, and `wm_page.get` parsed it without warning. Both links succeeded.

  **Instance 2 — error false on one write, and the other write was silently lost.** `wiki:specs:global-rules-cascade`: a content update (append a supersede note) was issued concurrently with a status update on the same page. The status update returned `IO_ERROR` — yet `status: superseded` **did** land. The content update, which returned success, **lost its write entirely**: the file ended with `status: superseded` but **no note**, and 73 bytes smaller (12,823 → 12,750 B). Re-issuing the content update **sequentially** succeeded — 13,932 B, note present — and returned no error.

  **Refined hypothesis.** The `IO_ERROR` ("No such file or directory") is a side-effect of concurrent same-page writes — plausibly a temporary-file or version-record collision — rather than a general path-resolution fault. The original "post-write bookkeeping" guess is consistent with this but less specific. The file body was never truncated in either instance: this is a lost update, not corruption.

  **The serious half is the lost update.** A concurrent write pair can discard one write while returning *either* `success` or `IO_ERROR`, so an agent cannot tell which write survived from the response. In instance 2 the *successful-looking* call was the one that lost. That makes the response non-authoritative in both directions at once — strictly worse than the false error alone.

  **Related.** `wiki:tasks:wmpagelink-concurrency-race-and-relates-to-clobber` (in-review, high) records a concurrency race around `wm_page.link` and `relates_to` clobbering. This addendum is the update-vs-update instance of the same class of defect; the two should be fixed and tested together rather than separately.

  **Workaround.** Never issue two writes to the same page concurrently; serialize page writes, and read the file back from disk afterwards per `@wiki/patterns/verify-writes-by-read-back`.
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