---
title: UX Polish — Post-Audit Fixes
type: spec
tags:
- spec
- ux
- polish
- web-ui
status: draft
relates_to:
  - {type: references, target: wiki:specs:sim-ui-polish}
---

schema_version: 1
state: |-
  id: wiki:specs:ux-polish

  ## UX Polish Spec — Post-Audit Fixes

  ### Context
  The designer completed a full UX principles audit (consistency, hierarchy, Gestalt, affordance, Fitts/Hick/Jakob laws). The audit found issues across all 6 views and the layout. Most were fixed inline; these remain as actionable tasks.

  ### Remaining Tasks

  #### P1 — High Impact
  | # | Task | Why | Views Affected |
  |---|------|-----|----------------|
  | 1 | Standardize page headers | Each view has a different header pattern (Graph = bar, Settings = heading+btn, rest = plain h1). Users expect consistent navigation landmarks. | Graph, Tasks, Pages, Memory, Settings, Search |
  | 2 | Dialog form validation | Create/Edit dialogs submit with empty fields. No error feedback, no disabled state. Hick's Law violation — form should guide user. | Pages, Memory |

  #### P2 — Medium Impact
  | # | Task | Why | Views Affected |
  |---|------|-----|----------------|
  | 3 | Add empty states | Tasks view shows nothing when board is empty. Users need to understand the system state. | Tasks, Graph |
  | 4 | Loading skeletons | Spinner-only doesn't communicate what's loading. Skeleton placeholders set expectation (Jakob's Law). | Search, Graph, Tasks, Pages, Memory, Settings |

  ### Acceptance Criteria

  For each task:
  - [ ] Changes follow Sim UI component conventions (wmBtn, wmInput, etc.)
  - [ ] Dark mode compatibility maintained
  - [ ] Build passes (`ng build`)
  - [ ] No regression in existing functionality

  ### Implementation Order
  1. Page headers (high impact, quick wins)
  2. Form validation (high impact, user-facing)
  3. Empty states (medium, low risk)
  4. Loading skeletons (medium, requires spartan skeleton component)

  ### References
  @wiki/tasks/befdeb
  @wiki/tasks/78a173
  @wiki/tasks/e9f569
  @wiki/tasks/b692f4
  @wiki/specs/sim-ui-polish
questions:
  - id: kind
    type: choice
    instructions: What kind of spec is this?
    options:
    - feature
    - system
    - doc
    - migration
    - experiment
  - id: scope
    type: choice
    instructions: How wide is the scope of this spec?
    options:
    - local
    - component
    - system
    - project-wide
  - id: status_class
    type: choice
    instructions: What lifecycle class is this spec in?
    options:
    - draft
    - reviewed
    - approved
    - superseded
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers: {}
