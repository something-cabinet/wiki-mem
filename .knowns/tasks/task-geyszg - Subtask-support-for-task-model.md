---
id: geyszg
title: Subtask support for task model
status: done
priority: medium
labels:
  - sprint-3
  - feature
  - tasks
createdAt: '2026-07-10T10:15:49.251Z'
updatedAt: '2026-07-10T11:58:54.904Z'
timeSpent: 141
assignee: '@me'
spec: specs/wm-leapfrog-replace-knowns-with-complete-memory-layer
fulfills:
  - AC-9
---
# Subtask support for task model

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add parent field to task frontmatter. wm_task.subtask tool to create child tasks linked to parent. Task board grouping by parent. Validation that subtasks inherit parent status constraints. Derived subtasks list at load time.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented subtask support: Added parent field to Frontmatter and WikiPageMeta. Added wm_task.subtask MCP tool that creates a child task with parent frontmatter, inherits parent tags, validates parent exists and is a task. All 170 tests pass.
<!-- SECTION:NOTES:END -->

