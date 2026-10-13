---
title: No Success Without a Verified Durable Write
type: rule
id: wiki:rules:durable-write-or-error
status: active
tags:
- rule
- tool-reliability
- honesty
- testing
- workflow
relates_to:
  - {type: references, target: wiki:specs:wmtask-checkac-persistence-fix}
---

schema_version: 1
state: |-
  ## Rule: No Success Without a Verified Durable Write

  A write operation on a WM tool (`wm_page.*`, `wm_task.*`, `wm_*`) MUST NOT return a success payload unless the change is durably persisted to the source of truth on disk. "The function returned `Ok`" is not evidence. If the write did not land, the tool MUST return an error.

  ### Why

  WM tools are the primary interface for autonomous agents. A success payload that lies about persistence corrupts workflow state invisibly: the board shows `todo` forever, acceptance criteria never tick, and version history disagrees with the page. Humans editing files directly never see these failures; agents build on them.

  ### How

  1. **Write, then verify.** After the write, re-read the file from disk and confirm the specific field(s) changed before returning success. Do not trust the in-memory graph, the version record, or the write helper's return value.
  2. **Fail loudly.** If the read-back does not match the requested change, return `ToolError::internal` (or `invalid_params` for bad input). Never return `{"status":"updated"}` on an unverified write.
  3. **Validate input before writing.** Bounds-check indices/ids and return an error for out-of-range input rather than succeeding silently.
  4. **Order side effects correctly.** Write derived stores (version records, indexes) only AFTER the source-of-truth write returns `Ok`. A version record must never claim a change the page does not have.
  5. **No-op branches must not report success.** A guard that returns the input unchanged (e.g. `if index == 0 { return input }`) is a silent lie when the caller asked for a change.

  ### Enforcement probes

  - Any write helper whose guard path can return the input unchanged without error is a violation: `rg -n "return yaml|return .*to_string\(\)" apps/wm-core/src/page/helpers`
  - Any handler that builds a success payload without a read-back: `rg -n "status: \"updated\"|\"checked\":|\"unchecked\":" apps/wm-core/src/mcp/tools`
  - Version/side-store writes that precede the page write: `rg -n "save_task_version|save_page_version" apps/wm-core/src -B2`

  ### Exceptions

  - Read-only operations (no persistence claim).
  - Pure in-memory computations with no durable backing store.

  ### Related

  - `@wiki/rules/tool-reliability-bug-tracking` — file a task when a tool lies
  - `@wiki/rules/no-compensating-layers` — do not paper over the broken layer with a retry/compensation layer
  - `@wiki/patterns/verify-writes-by-read-back` — the reusable implementation pattern
  - `@wiki/concepts/failure-silent-no-op-write-helper` — the motivating failure
questions:
  - id: enforcement
    type: choice
    instructions: How is this rule enforced?
    options:
    - ci-enforced
    - tool-enforced
    - review-enforced
    - convention-only
  - id: severity
    type: score
    instructions: How severe is a violation of this rule?
    levels:
    - advisory
    - recommended
    - required
    - blocking
  - id: has_exception
    type: noul
    instructions: This rule has documented exceptions.
  - id: applies_to
    type: choice
    instructions: Which surfaces does this rule apply to?
    multi: true
    options:
    - code
    - tests
    - docs
    - configuration
    - workflow
    - security
answers: {}