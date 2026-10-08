---
title: Add wm health audit CLI command skeleton
type: task
tags:
- from-spec
- spec:rebuild-log-findings
status: done
priority: high
acceptance_criteria:
  - text: "wm health audit CLI command exists with a HealthAction::Audit subcommand supporting --dry-run, --fix, and --format json|text flags"
  - text: "Audit command produces output formatting and a summary report"
---

schema_version: 1
state: |-
  Add Health variant to Commands enum with HealthAction::Audit subcommand supporting --dry-run, --fix, --format json|text flags. Includes output formatting and summary report.
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
