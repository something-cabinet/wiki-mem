---
title: ES-07 Migrate PageState
type: task
id: "wiki:tasks:es-07-migrate-pagestate"
status: done
priority: high
acceptance_criteria:
  - text: "PageState owns page_contents and content_hashes; no call site reads them off a sibling handle"
  - text: "page, graph and search no longer reference each other's fields directly, and the sibling-module import count drops measurably (AC-3)"
  - text: "A unit test proves PageState owns its state and does not reach into a sibling handle"
  - text: "The suites green at the first migration are still green (scenario 3)"
  - text: "cargo build, test and clippy --workspace are green with no new warnings"
tags: [from-spec, spec:engine-state-decomposition, refactor, wm-core, page]
implementation_notes: 'Implemented in 85dee7d. PageState owns page_contents and content_hashes privately; RwLock preserved rather than converted. Flat fields 7 -> 5. AC-3 recorded as 0 -> 0, not claimed as met: the edges were already closed by ES-05/ES-06, and both fields are dead (no product caller). Verified: cargo test --workspace exit=0, 42 suites.'
---

schema_version: 1
state: |-
  Seventh and last migration of @wiki/specs/engine-state-decomposition (D-7). Migrate `PageState`,
  owning `page_contents` and `content_hashes`.

  This is the highest-coupling step, deliberately last. On completion AC-3 must hold: `page`, `graph` and
  `search` no longer reference each other's fields directly, and the sibling-module import count drops
  measurably. Depends on ES-06.
  Part of @wiki/specs/engine-state-decomposition
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