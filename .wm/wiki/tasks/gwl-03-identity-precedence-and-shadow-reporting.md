---
title: GWL-03 Identity, project-wins precedence, shadow reporting
type: task
id: "wiki:tasks:gwl-03-identity-precedence-and-shadow-reporting"
status: cancelled
tags: [from-spec, spec:global-wiki-layer, engine, graph]
acceptance_criteria:
  - text: Same (type, slug) in both roots yields project-effective, global-shadowed, surfaced in wm_initial counts and audit output
  - text: Node identity is unchanged by precedence; existing page ids, refs and edges keep resolving with zero migration
priority: high
---

schema_version: 1
state: |-
  Implements D2, D3 and D11 with AC-2 of @wiki/specs/global-wiki-layer

  - Identity is `(type, slug)`. Scope is **provenance, not page data** — no frontmatter `scope` field, no id namespacing. A page from the global root is structurally identical to a project page. Consequences: zero migration, existing graphs and edges keep resolving, promotion is pure relocation.
  - When both roots supply the same identity, the project copy is effective and the global copy is shadowed.
  - Shadowing is **reported, never silent**: `wm_initial` prints `pages: P project, G global, S shadowed`.
  - This is what turns a false-confidence duplicate pair (e.g. `no-comments-in-code` at 498 B and 2149 B in different repos) into a single visible shadow report.
  - Edges resolve in the effective merged namespace by identity; a dangling global ref is an ordinary dangling ref caught by `wm-cli validate`.
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