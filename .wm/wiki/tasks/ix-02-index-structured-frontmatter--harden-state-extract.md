---
title: IX-02 Index structured frontmatter + harden state extract
type: task
id: "wiki:tasks:ix-02-index-structured-frontmatter--harden-state-extract"
status: done
priority: high
tags: [from-spec, spec:reinforce-indexing-flow, search-quality, p0]
spec: specs/reinforce-indexing-flow
acceptance_criteria:
  - text: "Structured frontmatter (FR/NFR/goals, AC, decision context/options/rationale, aliases) indexed into sections/BM25"
  - text: "record_state_text tolerates leading blanks/extra keys; never indexes raw YAML"
  - text: "A requirement phrase from a spec is findable via keyword search"
---

schema_version: 1
state: |-
  Index structured frontmatter fields into sections/docs and harden record_state_text. Per specs/reinforce-indexing-flow FR-2.
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
