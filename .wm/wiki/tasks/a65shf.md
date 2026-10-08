---
title: Add memory layers — project, global, session
type: task
status: done
tags: [feature, memory, knowns-parity]
priority: high
id: a65shf
acceptance_criteria:
  - text: "wm_memory.add accepts a layer parameter (project/global/session) and stores memory accordingly"
  - text: "Global memory is stored at ~/.wm/memory/ and session memory is ephemeral (not persisted)"
  - text: "A promote action moves project memory to global memory"
---

schema_version: 1
state: |-
  # Add memory layers — project, global, session

  > *Imported from Knowns task `a65shf`*

  # Add memory layers — project, global, session

  ## Description


  WM stores all memory flat in .wm/memory/. Knowns has 3 layers: project (project-scoped), global (~/.knowns/memory/), session (ephemeral working memory). Add layer parameter to wm_memory.add/get/list, add promote action to move project→global.


  ## Acceptance Criteria

  - [x] #1 wm_memory.add accepts layer parameter (project/global/session)
  - [x] #2 Global memory stored at ~/.wm/memory/
  - [x] #3 Session memory is ephemeral (not persisted)
  - [x] #4 promote action moves project→global memory
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
