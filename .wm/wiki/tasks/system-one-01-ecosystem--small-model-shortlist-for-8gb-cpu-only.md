---
title: system-one-01 Ecosystem + small-model shortlist for 8GB CPU-only
type: task
id: "wiki:tasks:system-one-01-ecosystem--small-model-shortlist-for-8gb-cpu-only"
status: done
priority: high
tags: [from-spec, spec:system-one-model-application, research, system-one]
spec: specs/system-one-model-application
acceptance_criteria:
  - text: "FR-1: System One decision-model paradigm explained and contrasted with WM's BM25/hybrid+RRF pipeline"
  - text: "FR-2/FR-4: each candidate named with params, on-disk size, peak RAM, license, runtime crate, and any benchmark/ECE"
  - text: "NFR-1/NFR-2/NFR-4: each candidate assessed for comfortable fit on an ~8 GB CPU-only machine; licenses permissive"
  - text: "FR-6: fine-tuning data format, minimum examples, local tooling, and export/swap path documented"
  - text: "All claims cite exact URLs; unknowns explicitly marked"
relates_to:
  - {type: relates_to, target: wiki:specs:system-one-model-application}
---

schema_version: 1
state: |-
  Research the local, free, Rust-runnable System One ecosystem and produce a candidate shortlist that runs in-process on a CPU-only ~8 GB RAM machine. Cover FR-1, FR-2, FR-4, FR-6 and NFR-1/2/4. No code changes.
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
