---
title: wm_task check_ac and status updates report success without persisting
type: task
id: "wiki:tasks:wmtask-checkac-and-status-updates-report-success-without-persisting"
status: todo
priority: urgent
tags: [bug, tool-reliability, task-store, urgent, sdd]
acceptance_criteria:
  - text: "wm_task check_ac writes checked: true into the task page frontmatter, verified by reading the file after the call"
  - text: "wm_task update with status and assignee persists both to the page, not only to the version record"
  - text: "A tool call that fails to persist returns an error instead of a success payload — no success response without a durable write"
  - text: "Version records are written only after the page write succeeds, so version history can never claim a change the page does not have"
  - text: "Regression test asserts page content after check_ac and after a status plus assignee update, reading the markdown from disk rather than the tool response"
---

Observed 2026-08-14 to 08-17 while running wm-flow on wiki:specs:code-edge-resolution. Two silent-write-loss bugs, both reporting success.

1. check_ac never persists. Six calls across two tasks all returned success payloads such as {"checked":[0]} yet no task page gained a checked field. Evidence: grep -c 'checked: true' on .wm/wiki/tasks/research-graphify-code-intel-edge-extraction-for-wm-adoption.md returns 0 after five successful check_ac calls, and the acceptance_criteria block of .wm/wiki/tasks/code-edge-resolution-01-refresh-the-code-index-at-the-write-path-and-via-the-watcher.md contains only text entries with no checked keys after a successful call on index 0. A second call on index 1 of the same task returned IO_ERROR No such file or directory.

2. Status and assignee updates can be lost while version history records them. wm_task update with status in-progress and assignee @me on task code-edge-resolution-01 returned {"status":"updated"}. The page still reads status: todo with no assignee. The version record .wm/versions/task-wiki-tasks-code-edge-resolution-01-...json v1 explicitly lists changes status todo to in-progress and assignee null to @me at 11:26:37Z, and v2 records implementation_plan which DID reach the page. So the same update path persists one field and drops others, and the version store is written even when the page write does not happen. Status updates to cancelled and in-review on other tasks in the same session did persist, so this is intermittent rather than total.

Impact for agents: an autonomous flow cannot record progress. The board shows todo forever, acceptance criteria never tick, and version history disagrees with the source of truth, so a later reader cannot tell which is right. This is the highest-severity class in wiki:rules:tool-reliability-bug-tracking because it corrupts the workflow state agents depend on, and it is invisible to humans who edit files directly.

Related prior art: memory wm_task stale for new pages — wm_page.update is the authoritative write prescribes wm_page.update as the workaround, which is what this session fell back to. Also related: wiki:tasks:wm-task-update-frontmatter-corruption (done) and wiki:tasks:four-sdd-workflow-tool-defects-found-during-spec-task-generation.

Also seen alongside: wm_time.start returns IO_ERROR No such file or directory yet writes time_started into the page, so the timer half-applies.