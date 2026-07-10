---
id: 7z1ctq
title: appendNotes mode for page and task updates
status: done
priority: high
labels:
  - sprint-0
  - feature
  - tasks
createdAt: '2026-07-10T10:14:42.363Z'
updatedAt: '2026-07-10T10:27:23.023Z'
timeSpent: 197
assignee: '@me'
spec: specs/wm-leapfrog-replace-knowns-with-complete-memory-layer
fulfills:
  - AC-3
---
# appendNotes mode for page and task updates

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add mode: "append" | "replace" parameter to wm_task.update and wm_page.update. Default remains "replace" for backward compatibility. append mode concatenates with newline separator instead of overwriting.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Add mode parameter to wm_task.update: "append" | "replace"
- [x] #2 Add mode parameter to wm_page.update: "append" | "replace"
- [x] #3 Default remains "replace" for backward compatibility
- [x] #4 Append mode concatenates with newline separator
- [x] #5 Add tests: append mode appends, replace mode overwrites, default is replace
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented: Added notes and append_notes fields to WmTaskUpdateInput and WmPageUpdateInput. Added implementation_notes + append_notes handling in page.rs update_page(). Added extract_yaml_string_value helper for reading existing notes from raw YAML. append mode concatenates with newline separator. Default behavior unchanged. All 148 tests pass.
<!-- SECTION:NOTES:END -->

