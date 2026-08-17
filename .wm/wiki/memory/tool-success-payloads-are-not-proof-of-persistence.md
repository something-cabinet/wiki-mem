---
title: Tool success payloads are not proof of persistence
type: memory
tags: [tool-reliability, task-store, verification, honesty]
status: active
---

A success payload from a wm tool is not proof of a durable write. Verify on disk.

Evidence 2026-08-17: `wm_task check_ac` returned `{"checked":[0]}` six times across two tasks and wrote zero `checked: true` to any page. `wm_task update` returned `{"status":"updated"}` for `status: in-progress` + `assignee: @me`, the page kept `status: todo`, and the version record at `.wm/versions/task-...json` v1 explicitly logged both field changes. `implementation_plan` persisted through that same call. So one call path persisted one field, dropped two, and wrote version history for all three — version history claiming a change the source of truth never received.

Practice: after any state-changing wm call whose result you will report or depend on, read the page back (grep the frontmatter key) before believing it. Prefer `wm_page.update` for task state — recorded in memory `wm_task stale for new pages` as the authoritative write, and it worked here when `wm_task update` did not.

Corollary for agent honesty: never report "task marked done" or "ACs checked" from a tool response alone. Put per-AC evidence in the notes body, which does persist, instead of relying on checkboxes that may not.

Tracked as wiki:tasks:wmtask-checkac-and-status-updates-report-success-without-persisting (urgent).