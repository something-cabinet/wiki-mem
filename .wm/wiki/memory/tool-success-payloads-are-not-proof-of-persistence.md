---
title: Tool success payloads are not proof of persistence
type: memory
tags: [tool-reliability, task-store, verification, honesty]
status: active
---



A success payload from a wm tool is not proof of a durable write. Verify on disk.

Evidence 2026-08-17: wm_task check_ac returned {"checked":[0]} six times across two tasks and wrote zero checked: true to any page. wm_task update returned {"status":"updated"} for status: in-progress + assignee: @me, the page kept status: todo, and the version record at .wm/versions/task-...json v1 explicitly logged both field changes. implementation_plan persisted through that same call.

Practice: after any state-changing wm call whose result you will report or depend on, read the page back (grep the frontmatter key) before believing it. Prefer wm_page.update for task state when a tool write path is suspect.

RESOLVED 2026-10-13: root cause was ac_set_checked (index==0 no-op guard + flush_pending marking inserted=true without pushing) plus handle_update writing the version record before the page write. Fixed in apps/wm-core (spec wiki:specs:wmtask-checkac-persistence-fix); write handlers now read the page back from disk and error if the change did not land. Pattern: @wiki/patterns/verify-writes-by-read-back Rule: @wiki/rules/durable-write-or-error