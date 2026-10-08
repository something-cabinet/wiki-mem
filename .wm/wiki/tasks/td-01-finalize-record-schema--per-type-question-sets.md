---
title: TD-01 Finalize record schema + per-type question sets
type: task
id: "wiki:tasks:td-01-finalize-record-schema--per-type-question-sets"
status: done
priority: high
tags: [from-spec, spec:typed-decision-doc-format, design, p0]
spec: specs/typed-decision-doc-format
acceptance_criteria:
  - text: "Envelope fields finalized with exact types and required/optional marks"
  - text: "Concrete question sets (ids, primitives, options/levels) for decision/rule/pattern/concept/howto/reference"
  - text: "Schema-version decision recorded"
  - text: "Open questions resolved: manifest location, runtime surface"
  - text: "Per-type question sets validated as being answerable/mappable by gliner-rs"
relates_to:
  - {type: relates_to, target: wiki:specs:typed-decision-doc-format}
---

schema_version: 1
state: |-
  Finalize the typed-decision record schema: exact envelope, per-type question sets for all six page types, schema versioning, and resolution of open questions (manifest location, runtime surface). Design only; orchestrator persists.
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
