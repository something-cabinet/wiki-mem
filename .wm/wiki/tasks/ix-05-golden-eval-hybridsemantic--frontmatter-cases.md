---
title: 'IX-05 Golden eval: hybrid/semantic + frontmatter cases'
type: task
id: "wiki:tasks:ix-05-golden-eval-hybridsemantic--frontmatter-cases"
status: done
priority: medium
tags: [from-spec, spec:reinforce-indexing-flow, testing]
spec: specs/reinforce-indexing-flow
acceptance_criteria:
  - text: "Golden eval de-ignored with hybrid/semantic + structured-frontmatter cases"
  - text: "Runs in CI/local"
---

schema_version: 1
state: |-
  Promote golden eval out of #[ignore]; add hybrid/semantic baselines + structured-frontmatter cases. Per specs/reinforce-indexing-flow FR-5.
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
