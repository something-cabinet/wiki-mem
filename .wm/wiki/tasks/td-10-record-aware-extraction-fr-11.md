---
title: TD-10 Record-aware extraction (FR-11)
type: task
id: "wiki:tasks:td-10-record-aware-extraction-fr-11"
status: done
priority: high
tags: [from-spec, spec:typed-decision-doc-format, engine, regression]
spec: specs/typed-decision-doc-format
acceptance_criteria:
  - text: "Record bodies derive search text/tags/refs from `state`, not raw YAML"
  - text: "BM25 heading boosts still work on state headings"
  - text: "`@wiki/` refs and inline tags extracted from state; non-record pages unchanged"
  - text: "Tests prove no search/graph regression on converted pages"
relates_to:
  - {type: relates_to, target: wiki:specs:typed-decision-doc-format}
---

schema_version: 1
state: |-
  FR-11: record-aware extraction so search/graph do not regress once page bodies are records. Section splitting, inline tags, and @wiki/ refs must read `state`. Prerequisite for the destructive apply (TD-06).
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
