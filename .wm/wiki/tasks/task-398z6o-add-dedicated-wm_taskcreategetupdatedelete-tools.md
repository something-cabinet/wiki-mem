---
title: Add dedicated wm_task.create/get/update/delete tools
type: task
status: done
tags: [feature, tasks, knowns-parity]
priority: medium
knowns_id: 398z6o
---

# Add dedicated wm_task.create/get/update/delete tools

> *Imported from Knowns task `398z6o`*

# Add dedicated wm_task.create/get/update/delete tools

## Description


Tasks are created through wm_page.create with type:task frontmatter. No dedicated task tools exist. Add wm_task.create/get/update/delete with task-specific field handling (status filtering, assignee, ACs, priority).


## Acceptance Criteria

- [x] #1 wm_task.create creates a wiki page with type:task
- [x] #2 wm_task.get reads task-specific fields
- [x] #3 wm_task.update updates task fields
- [x] #4 wm_task.delete removes a task page
