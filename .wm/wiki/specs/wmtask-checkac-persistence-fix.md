---
title: 'Spec: wm_task check_ac / task-state persistence fix'
type: spec
id: "wiki:specs:wmtask-checkac-persistence-fix"
status: draft
tags: [bugfix, tool-reliability, task-store, sdd]
relates_to:
  - {type: references, target: wiki:tasks:wmtask-checkac-and-status-updates-report-success-without-persisting}
functional_requirements:
  - {id: FR-1, description: "ac_set_checked inserts checked into the target acceptance_criteria item for multi-item lists; flush_pending marks inserted only when it actually pushes a line"}
  - {id: FR-2, description: "Index contract is 0-based and consistent across schema, handler and helper; schemars descriptions updated to 0-based"}
  - {id: FR-3, description: "Out-of-range index returns a ToolError instead of a success payload"}
  - {id: FR-4, description: "handle_check_ac/uncheck_ac read the page back and error if the criterion checked value did not change on disk"}
  - {id: FR-5, description: "handle_update persists the version record only after page.update_page returns Ok"}
  - {id: FR-6, description: "Regression tests assert the on-disk frontmatter after check_ac and after a status plus assignee update"}
non_functional_requirements:
  - {id: NFR-1, description: "No new comments in source (no-comments rule)"}
  - {id: NFR-2, description: "No workspace-wide cargo fmt; scope commands to -p wm-core"}
  - {id: NFR-3, description: "Zero compiler warnings (no-warnings rule)"}
general_goals:
  - description: "Restore durable AC and task-state writes so autonomous flows can record progress"
  - description: "Make the tool honest; no success payload without a durable, verified write"
---

schema_version: 1
state: |-
  # Spec: wm_task check_ac / task-state persistence

  ## Problem

  Two silent-write-loss defects, both reporting success. Tracked by
  wiki:tasks:wmtask-checkac-and-status-updates-report-success-without-persisting (urgent).
  Reproduced 2026-10-13 against HEAD v0.5.2-9-g76f99a9 and the running MCP binary (0.5.2).

  1. wm_task check_ac / uncheck_ac always return a success payload such as {"checked":[0]} but
     never write checked: true to the page. The page file is rewritten (a trailing newline is
     dropped) yet the acceptance_criteria frontmatter is byte-identical to the input.

  2. wm_task update writes the version record before the page write. A call reporting
     {"status":"updated"} can leave status/assignee absent from the page while the version file
     logs the change, so version history contradicts the source of truth.

  ## Root cause (verified by instrumentation)

  File: apps/wm-core/src/page/helpers/yaml_helper.rs, fn ac_set_checked (lines ~155-230).

  - Guard: "if index == 0 { return yaml.to_string(); }" makes index 0 a silent no-op while
    handle_check_ac returns success.

  - flush_pending sets *inserted = true even when indent is None. The first list item under
    acceptance_criteria flushes with target_indent == None (the target item is not yet known),
    which marks inserted = true and discards the insertion. When the real target item is later
    flushed, "!*inserted" is false, so nothing is written. Instrumented run: on the second item
    line the state was already inserted=true with no line pushed. Result: every index >= 1 also
    silently writes nothing.

  - The existing test page::tests::test_ac_set_checked is a false positive: its input already
    contains "checked: true" on a later item, so assertions on the whole string pass without the
    function doing anything.

  - Index base is ambiguous: the schemars descriptions say "(1-based)" while callers and incident
    evidence use 0-based. Either base, out-of-range indices succeed silently.

  File: apps/wm-core/src/mcp/tools/task/mod.rs, fn handle_check_ac / handle_uncheck_ac (lines
  ~655-681): build PageUpdateParams and return success unconditionally; no read-back, no error on
  out-of-range index.

  File: apps/wm-core/src/mcp/tools/task/mod.rs, fn handle_update: store.save_task_version(...) is
  called (line ~596) before page::update_page(...) (line ~598+), so version history is persisted
  before the page write is known to have succeeded.

  ## Design

  FR-1 ac_set_checked inserts checked into the target acceptance_criteria item for multi-item
  lists. flush_pending must only mark inserted when it actually pushes a line.

  FR-2 Index contract is 0-based and consistent across tool schema, handler and helper. index == 0
  targets the first criterion. The schemars descriptions for CheckAc/UncheckAc are updated to
  "0-based".

  FR-3 Out-of-range index (>= number of criteria, or no acceptance_criteria block) returns a
  ToolError instead of a success payload.

  FR-4 handle_check_ac / handle_uncheck_ac read the page back after the write and return an error
  if the target criterion's checked value did not change on disk.

  FR-5 handle_update persists the version record only after page::update_page returns Ok, so
  version history can never record a change the page does not have.

  FR-6 Regression tests assert the on-disk frontmatter (read the file, parse acceptance_criteria)
  after check_ac and after a status+assignee update, rather than asserting on the tool response or
  on the pre-existing string content.

  ## Files to change

  - apps/wm-core/src/page/helpers/yaml_helper.rs — flush_pending correctness; remove the index==0
    early return; 0-based indexing.
  - apps/wm-core/src/mcp/tools/task/action.rs — CheckAc/UncheckAc schemars descriptions (0-based).
  - apps/wm-core/src/mcp/tools/task/mod.rs — bounds check + read-back in check_ac/uncheck_ac; move
    save_task_version below the page write in handle_update.
  - apps/wm-core/src/page/mod.rs — fix test_ac_set_checked false positive; add block-style and
    index-0 coverage.

  ## Verification

  - cargo test -p wm-core (scoped; no workspace-wide cargo fmt).
  - A handler/integration test that writes a temp task page, calls the check path, and asserts the
    file's acceptance_criteria contains the expected checked value.
  - Manual: build the current binary and confirm check_ac(index) mutates the page on disk.

  ## Risks

  - Changing 1-based to 0-based is a behavior change; no other in-repo caller of check_ac exists
    (grep verified), so blast radius is limited to MCP clients, which now get an explicit error for
    an out-of-range index rather than silent success.
  - The read-back in FR-4 adds one file read per call; negligible for a local single-user tool.
questions:
  - id: spec_scope
    type: choice
    instructions: What is the scope of this spec?
    options:
    - feature
    - bugfix
    - refactor
  - id: needs_ac
    type: noul
    instructions: This spec has acceptance criteria in its task.
  - id: breaking_change
    type: noul
    instructions: This spec introduces a breaking change to a tool contract.
answers: {}