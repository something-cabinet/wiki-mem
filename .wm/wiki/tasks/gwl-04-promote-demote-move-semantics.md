---
title: GWL-04 promote/demote move semantics + collision refusal
type: task
id: "wiki:tasks:gwl-04-promote-demote-move-semantics"
status: cancelled
tags: [from-spec, spec:global-wiki-layer, cli, mcp]
acceptance_criteria:
  - text: promote uses move semantics; the graph node and edge set is identical before and after
  - text: A destination collision with differing content errors naming both paths plus a diff, writing zero bytes, unless force/rename is given
  - text: promote warns when the identity is shadowed in other project roots
priority: high
---

schema_version: 1
state: |-
  Implements D6 with AC-5 of @wiki/specs/global-wiki-layer

  - `promote` (to global) and `demote` (to project) use **move** semantics: write to the destination root, delete the origin, refresh derived state. Node identity is unchanged, so every `relates_to`/`extends`/`supersedes` edge survives — this is the argument against namespacing ids as `wiki:global:*`.
  - A destination collision with differing content **fails with both paths and a diff, writing zero bytes**, unless `force` or `rename` is given. This replaces the memory path's silent flatten-and-overwrite (`memory.rs:429-436`, where two projects promoting `my-title.md` collide).
  - `promote` also warns when the identity is shadowed in other project roots, so the owner sees the fork situation they are creating.
  - Test that the graph node and edge set is identical before and after a promote.
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