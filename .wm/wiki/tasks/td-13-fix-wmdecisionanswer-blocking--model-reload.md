---
title: TD-13 Fix wm_decision.answer blocking + model reload
type: task
id: "wiki:tasks:td-13-fix-wmdecisionanswer-blocking--model-reload"
status: done
priority: high
tags: [from-review, spec:typed-decision-doc-format, p1, runtime]
spec: specs/typed-decision-doc-format
acceptance_criteria:
  - text: "wm_decision.answer no longer blocks the MCP runtime thread (async + spawn_blocking)"
  - text: "The model backend is loaded once and cached, not per call"
  - text: "model_download streams/hashes files instead of buffering ~300MB into memory"
  - text: "Tests updated/added; cargo check/clippy clean"
---

schema_version: 1
state: |-
  Review P1-1: wm_decision.answer blocks the MCP server and reloads the ~300MB model per call. Make it async (register_typed_async + spawn_blocking), cache the backend (keyed by model dir + manifest revision), and stream-hash downloads.
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
