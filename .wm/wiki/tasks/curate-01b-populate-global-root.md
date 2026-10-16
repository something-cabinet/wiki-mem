---
title: '[CURATE-01b] Populate global root, shadow decisions, audit verification'
type: task
id: "wiki:tasks:curate-01b-populate-global-root"
status: cancelled
tags: [curation, rules, global, blocked]
acceptance_criteria:
  - text: Canonical set exists at the global rules root, created via the global write surface rather than direct file writes
  - text: Collision refusal exercised on the same-name-different-content pairs, with no silent overwrite
  - text: Per-project shadow decisions recorded, with idle-incremental-game behaviour stated explicitly
  - text: Global audit reports zero same-slug-different-SHA across scopes and an empty name-cluster report
priority: high
relates_to:
  - {type: depends_on, target: wiki:tasks:curate-01-canonical-global-rule-set}
---

schema_version: 1
state: |-
  Phase B of CURATE-01; phase A is the parent task.

  **Blocked on the composition mechanism.**

  5. Populate the global rules root through the global write/promote surface, not by direct file writes. Exercise collision refusal on the same-name-different-content pairs (no-comments-in-code 498 B vs 2149 B; no-magic-values 508 B vs 3190 B).

  6. Shadow decisions per project; record idle-incremental-game behaviour explicitly.

  7. Verify with the global audit command: zero same-slug-different-SHA across scopes and an empty name-cluster report.

  Depends on gwl-01, gwl-02, gwl-04, gwl-05.

  Sequencing, owner-approved 2026-10-15: the mechanism ships with an empty global root, which binds nothing, and curation populates it afterwards. This reverses the superseded cascade spec requirement that curation land by the time the cascade ships. The empty-first order satisfies that intent more safely, since the risk only materialises if the mechanism ships with uncurated content already in the global root.
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