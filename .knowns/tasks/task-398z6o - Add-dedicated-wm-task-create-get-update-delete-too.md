---
id: 398z6o
title: Add dedicated wm_task.create/get/update/delete tools
status: todo
priority: medium
labels:
  - feature
  - tasks
  - knowns-parity
createdAt: '2026-07-08T11:16:26.576Z'
updatedAt: '2026-07-08T11:16:26.576Z'
timeSpent: 0
---
# Add dedicated wm_task.create/get/update/delete tools

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Tasks are created through wm_page.create with type:task frontmatter. No dedicated task tools exist. Add wm_task.create/get/update/delete with task-specific field handling (status filtering, assignee, ACs, priority).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 wm_task.create creates a wiki page with type:task
- [ ] #2 wm_task.get reads task-specific fields
- [ ] #3 wm_task.update updates task fields
- [ ] #4 wm_task.delete removes a task page
<!-- AC:END -->

