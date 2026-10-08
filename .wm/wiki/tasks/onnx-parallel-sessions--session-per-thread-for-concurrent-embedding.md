---
id: wiki:tasks:onnx-parallel-sessions--session-per-thread-for-concurrent-embedding
title: ONNX Parallel Sessions — Session-per-thread for concurrent embedding
type: task
status: done
priority: medium
tags:
- from-spec
- spec:onnx-incremental-and-optimization
spec: specs/onnx-incremental-and-optimization
acceptance_criteria:
- text: 'AC-4: Rebuild with 1,000 sections uses all CPU threads for ONNX inference'
- text: 'NFR-2: Memory stays under 2 GB total'
---

schema_version: 1
state: |-
  id: wiki:tasks:onnx-parallel-sessions--session-per-thread-for-concurrent-embedding

  FR-4: Replace Mutex<Session> with thread-local ONNX sessions. Each rayon worker creates its own session for fully parallel embedding.
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
