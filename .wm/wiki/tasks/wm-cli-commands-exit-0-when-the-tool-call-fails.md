---
title: wm-cli commands exit 0 when the tool call fails
type: task
id: "wiki:tasks:wm-cli-commands-exit-0-when-the-tool-call-fails"
status: todo
priority: high
tags: [cli, tool-reliability]
spec: wiki:specs:cli-commands-must-exit-non-zero-when-the-tool-fails
acceptance_criteria:
  - text: Every call site that swallows a tool error now propagates it and the process exits 1
  - text: Success output of every command is unchanged and the golden characterization cases pass
  - text: Each command family has a CLI test asserting a non-zero exit and the error on stderr
---

schema_version: 1
state: |-
  29 call sites in 10 files under apps/wm-cli/src print the tool error with eprintln and return Ok, so the process exits 0 (graph, page, search, source, time and others). index rebuild and index embed are already fixed. See the spec.
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
