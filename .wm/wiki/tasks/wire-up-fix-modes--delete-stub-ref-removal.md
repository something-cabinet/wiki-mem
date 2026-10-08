---
title: Wire up fix modes — delete, stub, ref removal
type: task
tags:
- from-spec
- spec:rebuild-log-findings
status: done
priority: high
acceptance_criteria:
- text: --fix mode deletes stale empty task pages
- text: --fix mode stubs active task pages with a description
- text: --fix mode removes broken relates_to entries from YAML frontmatter
---

schema_version: 1
state: |-
  Implement --fix mode: delete stale empty task pages, stub active ones with description, remove broken relates_to entries from YAML frontmatter.
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
