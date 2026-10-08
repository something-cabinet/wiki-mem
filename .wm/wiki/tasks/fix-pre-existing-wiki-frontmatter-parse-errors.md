---
title: Fix pre-existing wiki frontmatter parse errors
type: task
id: wiki:tasks:fix-pre-existing-wiki-frontmatter-parse-errors
status: todo
priority: low
tags: [bug, wiki-health, frontmatter]
acceptance_criteria:
  - text: "wm lint check reports no frontmatter parse errors for linus-core-simplicity-rule and graph-index-staleness-write-handlers-need-disk-fallback"
  - text: "Both pages parse cleanly with correct typed fields"
---

schema_version: 1
state: |-
  Pre-existing wiki parse errors surfaced by the CLI task board during the wm-doc-fix wave: (1) .wm/wiki/specs/linus-core-simplicity-rule.md - general_goals[0] invalid type: string, expected struct GoalEntry; (2) .wm/wiki/memory/graph-index-staleness-write-handlers-need-disk-fallback.md - mapping values not allowed (unquoted value with colon). Both break frontmatter parsing; fix the source pages (via wm_page update, never manual edit).
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
