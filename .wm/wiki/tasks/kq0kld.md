---
title: Semantic Search E2E Tests (opt-in)
type: task
status: done
tags: [test, semantic, onnx]
priority: low
id: kq0kld
acceptance_criteria:
  - text: "wm-core/tests/semantic_test.rs exists behind the #[cfg(feature = \"embed\")] gate"
  - text: "Semantic search query returns results and hybrid search RRF fusion is tested (including AC-E19 model switch cleanup)"
  - text: "Graceful degradation is verified when the model is absent"
---

schema_version: 1
state: |-
  # Semantic Search E2E Tests (opt-in)

  > *Imported from Knowns task `kq0kld`*

  # Semantic Search E2E Tests (opt-in)

  ## Description


  Create wm-core/tests/semantic_test.rs behind #[cfg(feature = "embed")] gate: download model, index pages, test semantic search query returns results, test hybrid search RRF fusion, test model switch cleanup (AC-E19), test graceful degradation when model absent. Requires ONNX Runtime + model binary.


  ## Acceptance Criteria
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
