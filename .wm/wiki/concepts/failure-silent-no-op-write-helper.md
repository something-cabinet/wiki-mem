---
title: 'Failure: Silent no-op write helper'
type: concept
id: wiki:concepts:failure-silent-no-op-write-helper
status: active
tags: [concept, failure-analysis, tool-reliability, yaml, testing]
relates_to:
  - {type: relates_to, target: wiki:patterns:verify-writes-by-read-back}
---

schema_version: 1
state: |-
  ## What went wrong

  `wm_task check_ac` returned a success payload (`{"checked":[0]}`) on every call yet wrote zero `checked: true` to any page. Six calls across two tasks produced no change. The failure was invisible through the tool API — `wm_page.get` returned the response shape callers expected, and the file on disk was byte-identical in its `acceptance_criteria` block after the call.

  Two defects compounded:

  1. `ac_set_checked` (apps/wm-core/src/page/helpers/yaml_helper.rs) had an `if index == 0 { return yaml.to_string(); }` guard — index 0 was a silent no-op.
  2. `flush_pending` set `*inserted = true` even when `indent` was `None`. The first list item under `acceptance_criteria` flushed before the target item's indent was known, marking `inserted = true` and discarding the insertion; the later real flush was skipped by `if !*inserted`. Net effect: for a multi-item list, NO index ever wrote. Instrumentation confirmed the second item line already carried `inserted = true` with nothing pushed.

  A third defect hid the first two: `test_ac_set_checked` was a false positive. Its fixture already contained `checked: true` on a later item, so `assert!(result.contains("checked: true"))` passed even though the helper inserted nothing.

  A related ordering defect lived in `handle_update`: `store.save_task_version(...)` ran before `page::update_page(...)`, so the version record could claim a status/assignee change the page never received.

  ## Root cause

  A write helper that can return its input unchanged on a valid call, combined with a caller that treats "the function returned `Ok`" as proof of persistence. The success payload described the request, not the result. Tests asserted on pre-existing string content instead of the produced artifact.

  ## Prevention

  - Read the file back from disk after a write and assert the specific field changed; return an error otherwise. See `@wiki/patterns/verify-writes-by-read-back` and rule `@wiki/rules/durable-write-or-error`.
  - No-op/guard branches must return an error when a change was requested.
  - Bounds-check index/id inputs; out-of-range must error, not succeed silently.
  - Order derived stores (version, index) after the source-of-truth write.
  - Assert on the artifact: write a real page, call the tool, re-read and parse the markdown. Never assert `contains(...)` against content that the fixture already provided.

  ## Time lost

  Roughly three sessions of agent workflow (2026-08-14 to 2026-08-17) where progress could not be recorded — acceptance criteria never ticked and the board stayed `todo` while version history disagreed with the page.

  ## Related

  - @wiki/tasks/wmtask-checkac-and-status-updates-report-success-without-persisting
  - @wiki/patterns/verify-writes-by-read-back
  - @wiki/rules/durable-write-or-error
questions:
  - id: kind
    type: choice
    instructions: What kind of concept document is this?
    options:
    - concept
    - failure-analysis
    - research-report
    - reference-note
  - id: category
    type: choice
    instructions: Which domain category does this concept belong to?
    options:
    - architecture
    - search-retrieval
    - graph
    - parser-format
    - mcp-tooling
    - cli
    - storage
    - embeddings
    - web-ui
    - process
  - id: maturity
    type: score
    instructions: How mature is the understanding of this concept?
    levels:
    - raw
    - exploratory
    - established
    - stable
  - id: code_referenced
    type: noul
    instructions: This concept references concrete code.
answers: {}