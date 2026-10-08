---
title: 'P0: Wire body @wiki/ references into graph builder'
id: fbe6a0
type: task
status: done
priority: high
tags:
- from-spec
- spec:graph-connectivity-fix
- p0
acceptance_criteria:
- text: build_graph_from_wiki calls reference_service::extract_references(body) after collecting frontmatter relates_to entries
- text: Body @wiki/ references are converted to edges, deduplicated, and reciprocal reference edges are added (FR-1 to FR-4)
---

schema_version: 1
state: |-
  Implement FR-1 to FR-4. In build_graph_from_wiki, after collecting frontmatter relates_to entries, call reference_service::extract_references(body) to extract body @wiki/ refs, convert to edges, deduplicate. Add reciprocal references edges for each body-extracted ref.
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
