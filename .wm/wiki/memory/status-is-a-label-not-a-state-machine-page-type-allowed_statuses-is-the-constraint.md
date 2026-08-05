---
title: Status is a label, not a state machine; page-type allowed_statuses is the constraint
type: memory
tags: [status, task, page-status-model, bugfix]
status: active
---

Task/page status is a label, not a state machine. `PageStatus::can_transition_to()` is permissive (always Ok) so any status can be set at any time (e.g. todo -> done directly). The real constraint is page-type scoped: `PageType::allowed_statuses()` (Task: todo, in-progress, in-review, done, blocked, cancelled). Enforcement sites: wm_task.update (task/mod.rs), wm_task.subtask, wm_page.update/create (page/mod.rs + page_update_builder_service.rs). Regression tests: test_wm_task_update_todo_to_done, test_wm_task_update_rejects_non_task_status (mcp_test.rs), test_todo_can_go_directly_to_done (page_status_model.rs).