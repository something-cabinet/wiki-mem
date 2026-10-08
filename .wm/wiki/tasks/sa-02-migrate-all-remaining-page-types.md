---
title: SA-02 Migrate all remaining page types
type: task
id: "wiki:tasks:sa-02-migrate-all-remaining-page-types"
status: done
priority: high
tags: [from-spec, spec:structured-all-doc-types, migration, destructive]
spec: specs/structured-all-doc-types
acceptance_criteria:
  - text: "Migration converts all remaining page types (except index.md)"
  - text: "Per-type counts reported; 0 parse failures; frontmatter byte-preserved"
  - text: "Re-run is a no-op"
  - text: "Rule/memory/task prose preserved in state"
---

schema_version: 1
state: |-
  Extend the migration to all page types and convert the remaining pages (rule/core/memory/task/spec/note). Depends on SA-01. Per specs/structured-all-doc-types.
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