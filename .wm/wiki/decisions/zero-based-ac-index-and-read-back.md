---
title: 'Decision: Zero-based AC index and read-back on task writes'
type: decision
id: wiki:decisions:zero-based-ac-index-and-read-back
status: approved
tags: [decision, tool-reliability, api-design, task-store]
relates_to:
  - {type: references, target: wiki:tasks:wmtask-checkac-and-status-updates-report-success-without-persisting}
  - {type: implements, target: wiki:specs:wmtask-checkac-persistence-fix}
---

schema_version: 1
state: |-
  ## Context

  `wm_task check_ac`/`uncheck_ac` took an `index` documented as "1-based" while callers and incident evidence used 0-based, the helper had an `index == 0` no-op guard, and neither base produced a durable write (see `@wiki/concepts/failure-silent-no-op-write-helper`). Index 0 returned success and wrote nothing; out-of-range indices also returned success. The tool also returned `{"status":"updated"}` before the version record and page could disagree.

  ## Decision

  1. The acceptance-criterion index is **0-based** and consistent across the schemars schema, the handler, and the helper. `index == 0` targets the first criterion.
  2. Out-of-range index (>= number of criteria, or no `acceptance_criteria` block) returns `ToolError::invalid_params` — never a success payload.
  3. `handle_check_ac`/`uncheck_ac` re-read the page from disk and verify the target criterion's `checked` value changed before returning success.
  4. `handle_update` writes the version record only after `page::update_page(...)` returns `Ok`, and verifies `status`/`assignee` on disk when supplied.

  ## Rationale

  - **0-based over 1-based:** the value is a `usize` index into a `Vec`, serde and Rust idiom is 0-based, and every observed caller thought in 0-based. The old "1-based with a 0 sentinel" was the source of the silent no-op — removing the sentinel removes the failure mode. No other in-repo caller of `check_ac` exists (grep-verified), so the blast radius is MCP clients, which now get an explicit error for a bad index instead of a lie.
  - **Read-back over trusting `Ok`:** the write helper's return value is not evidence of persistence; the source of truth is the file. Cost is one file read per call — negligible for a local single-user tool.
  - **Version after page:** a version record must never claim a change the page does not have.

  ## Consequences

  - MCP clients passing 1-based indices now get `INVALID_PARAMS` instead of silently hitting the wrong criterion or a no-op — a breaking change, accepted as the fix for the silent-success class.
  - All write paths for ACs and task state carry an on-disk verification step; the `check_ac` success payload is now durable-verified.
  - Regression coverage reads the markdown from disk: 10 unit tests (`page::tests::test_ac_set_checked_*`) and 3 in-process integration tests in `apps/wm-core/tests/mcp_test.rs`.

  ## Related

  - @wiki/tasks/wmtask-checkac-and-status-updates-report-success-without-persisting
  - @wiki/specs/wmtask-checkac-persistence-fix
  - @wiki/patterns/verify-writes-by-read-back
  - @wiki/rules/durable-write-or-error
questions:
  - id: outcome
    type: choice
    instructions: What is the recorded outcome of this decision?
    options:
    - adopted
    - rejected
    - deferred
    - superseded
    - abandoned
  - id: reversibility
    type: noul
    instructions: The decision can be reversed cheaply without data migration or cross-module breakage.
  - id: confidence
    type: score
    instructions: How strong is the recorded justification for the selected outcome?
    levels:
    - low
    - medium
    - high
  - id: impact
    type: choice
    instructions: How wide is the blast radius of this decision?
    options:
    - local
    - component
    - system
    - project-wide
answers: {}