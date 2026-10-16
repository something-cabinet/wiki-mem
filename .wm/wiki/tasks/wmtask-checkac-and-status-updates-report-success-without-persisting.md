---
title: wm_task check_ac and status updates report success without persisting
type: task
id: "wiki:tasks:wmtask-checkac-and-status-updates-report-success-without-persisting"
status: in-review
priority: urgent
tags: [bug, tool-reliability, task-store, urgent, sdd]
implementation_notes: |-
  Root-caused and fixed in apps/wm-core. ac_set_checked: removed the index==0 silent no-op and the flush_pending inserted=true-without-push defect; 0-based indexing. handle_check_ac/uncheck_ac: bounds check + on-disk read-back. handle_update: version record now written after the page write, with status/assignee read-back. All 413 non-ONNX wm-core tests pass. Pending: commit + rebuild/install the MCP binary (running binary is 0.5.2 and still exhibits the bug).

  ## 2026-10-15 — check_ac still a silent no-op (regression after a776edb)

  Reproduced while marking ACs on `wiki:tasks:wiki-tool-reliability-wmdocget-core-doc-bootstrap-guidance--id-vs-path-plus-core-session-loading`.

  - Input: `wm_task({"action":"check_ac","id":"<task>","index":0})` … `index:4` (5 calls, all valid indices)
  - Output: `{"checked":[0]}` … `{"checked":[4]}` — success echo per call
  - Counter-evidence: `wm_task({"action":"get","id":"<task>"})` returns `checked: false` for **all five** ACs; raw read of the task file shows **zero checkbox markers** and `status: in-progress`.
  - Root-cause hypothesis: `check_ac` still writes the legacy `- [ ] text` checkbox form, but task ACs are now record-schema entries (`acceptance_criteria: - text: "…"`), so the write target no longer exists and the success payload is a lie. Violates rule `durable-write-or-error`.
  - Workaround: none via MCP. AC state is not persisted and the task cannot be closed by AC completion.

  Also note commit `a776edb fix(task): make check_ac writes persist` appears to have been re-broken by the record-format migration (d0e8347 / 76f99a9).
  Reproduced 2026-10-16 at HEAD e74e935 through the MCP wm_task tool.

  check_ac was called for indices 0, 1, 2 and 3 on wiki:tasks:es-01-scaffold-the-handle-facade-and-migrate-auditstate.
  Each call returned success, for example {"checked":[0],"id":"wiki:tasks:es-01-scaffold-the-handle-facade-and-migrate-auditstate"}.

  Reading the file back from disk shows no checked key on any of the four acceptance criteria - all four are still
  unchecked. A scan of .wm/wiki/tasks/*.md finds zero occurrences of 'checked: true' workspace-wide, so no
  acceptance criterion has ever persisted as checked in this wiki.

  The two halves of this task behave differently: wm_task.update DOES persist status, since the same update call
  set status: done and the file shows status: done. Only the check_ac half fails to persist.

  ## 2026-10-15 — check_ac still a silent no-op (regression after a776edb)

  Reproduced while marking ACs on `wiki:tasks:wiki-tool-reliability-wmdocget-core-doc-bootstrap-guidance--id-vs-path-plus-core-session-loading`.

  - Input: `wm_task({"action":"check_ac","id":"<task>","index":0})` … `index:4` (5 calls, all valid indices)
  - Output: `{"checked":[0]}` … `{"checked":[4]}` — success echo per call
  - Counter-evidence: `wm_task({"action":"get","id":"<task>"})` returns `checked: false` for **all five** ACs; raw read of the task file shows **zero checkbox markers** and `status: in-progress`.
  - Root-cause hypothesis: `check_ac` still writes the legacy `- [ ] text` checkbox form, but task ACs are now record-schema entries (`acceptance_criteria: - text: "…"`), so the write target no longer exists and the success payload is a lie. Violates rule `durable-write-or-error`.
  - Workaround: none via MCP. AC state is not persisted and the task cannot be closed by AC completion.

  Also note commit `a776edb fix(task): make check_ac writes persist` appears to have been re-broken by the record-format migration (d0e8347 / 76f99a9).
acceptance_criteria:
  - text: "wm_task check_ac writes checked: true into the task page frontmatter, verified by reading the file after the call"
    checked: true
  - text: "wm_task update with status and assignee persists both to the page, not only to the version record"
    checked: true
  - text: "A tool call that fails to persist returns an error instead of a success payload — no success response without a durable write"
    checked: true
  - text: "Version records are written only after the page write succeeds, so version history can never claim a change the page does not have"
    checked: true
  - text: "Regression test asserts page content after check_ac and after a status plus assignee update, reading the markdown from disk rather than the tool response"
    checked: true
---

schema_version: 1
state: |-
  Observed 2026-08-14 to 08-17 while running wm-flow on wiki:specs:code-edge-resolution. Two silent-write-loss bugs, both reporting success.

  1. check_ac never persists. Six calls across two tasks all returned success payloads such as {"checked":[0]} yet no task page gained a checked field. Evidence: grep -c 'checked: true' on .wm/wiki/tasks/research-graphify-code-intel-edge-extraction-for-wm-adoption.md returns 0 after five successful check_ac calls, and the acceptance_criteria block of .wm/wiki/tasks/code-edge-resolution-01-refresh-the-code-index-at-the-write-path-and-via-the-watcher.md contains only text entries with no checked keys after a successful call on index 0. A second call on index 1 of the same task returned IO_ERROR No such file or directory.

  2. Status and assignee updates can be lost while version history records them. wm_task update with status in-progress and assignee @me on task code-edge-resolution-01 returned {"status":"updated"}. The page still reads status: todo with no assignee. The version record .wm/versions/task-wiki-tasks-code-edge-resolution-01-...json v1 explicitly lists changes status todo to in-progress and assignee null to @me at 11:26:37Z, and v2 records implementation_plan which DID reach the page. So the same update path persists one field and drops others, and the version store is written even when the page write does not happen. Status updates to cancelled and in-review on other tasks in the same session did persist, so this is intermittent rather than total.

  Impact for agents: an autonomous flow cannot record progress. The board shows todo forever, acceptance criteria never tick, and version history disagrees with the source of truth, so a later reader cannot tell which is right. This is the highest-severity class in wiki:rules:tool-reliability-bug-tracking because it corrupts the workflow state agents depend on, and it is invisible to humans who edit files directly.

  Related prior art: memory wm_task stale for new pages — wm_page.update is the authoritative write prescribes wm_page.update as the workaround, which is what this session fell back to. Also related: wiki:tasks:wm-task-update-frontmatter-corruption (done) and wiki:tasks:four-sdd-workflow-tool-defects-found-during-spec-task-generation.

  Also seen alongside: wm_time.start returns IO_ERROR No such file or directory yet writes time_started into the page, so the timer half-applies.

  Resolution 2026-10-13 (verified in source + tests, HEAD v0.5.2-9-g76f99a9): see
  wiki:specs:wmtask-checkac-persistence-fix. Two root causes isolated by instrumentation. (1)
  ac_set_checked had an "if index == 0 return yaml" no-op and flush_pending set inserted = true even
  when indent was None, so the first item flush discarded the insertion and every index wrote
  nothing; fixed as a 0-based state machine. (2) handle_update wrote the version record before the
  page write; reordered, and status/assignee are now read back from disk. handle_check_ac/uncheck_ac
  now bounds-check the index (out of range -> INVALID_PARAMS) and verify the on-disk result before
  returning success. Regression coverage: 10 unit tests (page::tests::test_ac_set_checked_*) and 3
  in-process integration tests in apps/wm-core/tests/mcp_test.rs that re-read the markdown from disk.
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