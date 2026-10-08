---
title: 'TD-07 Runtime: gliner-rs mapping + model manifest'
type: task
id: "wiki:tasks:td-07-runtime-gliner-rs-mapping--model-manifest"
status: done
priority: medium
tags: [from-spec, spec:typed-decision-doc-format, runtime]
spec: specs/typed-decision-doc-format
acceptance_criteria:
  - text: "Record state/questions map into the model wire format"
  - text: "Runtime answers via gliner-rs offline"
  - text: "Model obtained via checksum-pinned manifest; not committed"
relates_to:
  - {type: relates_to, target: wiki:specs:typed-decision-doc-format}
---

schema_version: 1
state: |-
  Runtime integration: map record -> gliner-rs input, run choice/score/noul via its probability API, and add the model manifest + checksum-pinned download. Depends on TD-01.
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
