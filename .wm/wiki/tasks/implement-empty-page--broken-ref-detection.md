---
title: Implement empty page + broken ref detection
type: task
tags:
- from-spec
- spec:rebuild-log-findings
status: done
priority: high
acceptance_criteria:
- text: Health audit scans wiki pages and detects empty tasks (zero parseable sections)
- text: Health audit scans YAML frontmatter and reports relates_to targets that don't exist in the graph
- text: Detection results are surfaced in the health audit output with affected page references
---

schema_version: 1
state: |-
  Implement health audit detection logic: scan wiki pages for zero parseable sections (empty tasks), and scan YAML frontmatter for relates_to targets that don't exist in the graph.
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
