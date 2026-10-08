---
id: wiki:tasks:onnx-cursor-scanning----since-flag-for-incremental-rebuild
title: ONNX Cursor Scanning — --since flag for incremental rebuild
type: task
status: done
priority: medium
tags:
- from-spec
- spec:onnx-incremental-and-optimization
spec: specs/onnx-incremental-and-optimization
acceptance_criteria:
- text: 'AC-8: --since 2026-07-01 only processes sections modified after that date'
---

schema_version: 1
state: |-
  id: wiki:tasks:onnx-cursor-scanning----since-flag-for-incremental-rebuild

  FR-7: Add --since <timestamp> flag to wm index rebuild CLI command. Filter sections by page metadata updated_at.
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
