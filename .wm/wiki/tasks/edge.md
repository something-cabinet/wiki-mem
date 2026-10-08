---
id: wiki:tasks:edge
title: "CLOSED — Register custom edge type 'implemented-by' in config"
type: task
status: cancelled
spec: specs/edge-type-pruning
superseded_by: wiki:specs:edge-type-pruning
relates_to:
  - {type: implements, target: wiki:specs:edge-type-pruning}
acceptance_criteria:
  - text: "No \"Custom edge type 'implemented-by' not registered in config\" warning would remain"
  - text: "The 2 implemented-by edges would be represented in the graph via implements edges from the decision side per the inverse-edge policy"
---

schema_version: 1
state: |-
  id: wiki:tasks:edge

  **Severity:** Low

  **Resolution:** Won't-do per edge-type-pruning spec. Inverse-edge policy: canonical single direction + reverse traversal. The 2 `implemented-by` edges were rewritten as `implements` from the decision side (see decisions/init-setup-separation, decisions/error-response-format).

  **Acceptance Criteria:**
  - [ ] No "Custom edge type 'implemented-by' not registered in config" warning
  - [ ] `implemented-by` edges appear in graph
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
