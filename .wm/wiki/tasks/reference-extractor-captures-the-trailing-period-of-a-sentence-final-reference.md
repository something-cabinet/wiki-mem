---
title: Reference extractor captures the trailing period of a sentence-final reference
type: task
id: "wiki:tasks:reference-extractor-captures-the-trailing-period-of-a-sentence-final-reference"
status: done
priority: medium
tags: [references, graph, parser]
spec: wiki:specs:reference-extractor-must-not-capture-trailing-periods
acceptance_criteria:
  - text: A sentence-final @wiki reference resolves to the page it names
    checked: true
  - text: A name containing a period inside it still resolves
    checked: true
  - text: Unit tests cover sentence-final, comma-followed, parenthesised and dotted-name references
    checked: true
implementation_notes: 'Fixed in apps/wm-core/src/reference_service.rs: the captured target is trimmed of trailing periods and an all-period target is ignored. Six unit tests written first, four failed before the change. A period inside a name is kept.'
---

schema_version: 1
state: |-
  REFERENCE_RE in apps/wm-core/src/reference_constant.rs allows a period in the target, so a reference that ends a sentence resolves to the page name plus a period and the graph logs unresolved relates_to. Seen as 5 unresolved targets in eightcap-new-portal and 1 in the wm-mem persist spec. See the spec.
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