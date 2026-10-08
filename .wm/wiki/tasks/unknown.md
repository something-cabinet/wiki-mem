---
id: wiki:tasks:unknown
title: Fix 4 wiki pages with unknown status 'active'
type: task
status: cancelled
acceptance_criteria:
  - text: "No \"Unknown page status string: 'active'\" warnings would be emitted on startup"
  - text: "All 4 affected wiki pages would use valid status values (draft, reviewed, approved, done, in-progress, todo)"
---

schema_version: 1
state: |-
  id: wiki:tasks:unknown

  **Severity:** Low

  **Observed:** 4 wiki pages use status 'active' which isn't recognized by the parser. They fall back to 'draft'.

  **Root Cause:** Pages with `status: active` in their frontmatter. Valid statuses are: draft, reviewed, approved, done, in-progress, todo.

  **Files affected:** Check `.wm/wiki/` for pages with `status: active`.

  **Acceptance Criteria:**
  - [ ] No "Unknown page status string: 'active'" warnings on startup
  - [ ] All pages use valid status values
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
