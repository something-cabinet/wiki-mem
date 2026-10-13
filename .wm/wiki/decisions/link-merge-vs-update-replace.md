---
title: 'Decision: link merges relates_to, update replaces'
type: decision
id: wiki:decisions:link-merge-vs-update-replace
status: approved
tags: [decision, api-design, frontmatter, graph]
relates_to:
  - {type: references, target: wiki:tasks:wmpagelink-concurrency-race-and-relates-to-clobber}
  - {type: implements, target: wiki:specs:wmpagelink-merge-and-write-serialization}
---

schema_version: 1
state: |-
  ## Context

  `wm_page.link` built `PageUpdateParams { relates_to: Some(vec![ single new edge ]) }`, and `apply_relates_to` removed the whole `relates_to` block before writing that list. So each `link` call destroyed every existing edge — including edges authored directly in frontmatter — while returning `{"status":"linked"}`. The same `relates_to` parameter is also used by `wm_page.update`, where replacement is the documented behaviour.

  ## Decision

  - `wm_page.link` has **merge** semantics: it reads the on-disk `relates_to`, appends the new `(type, target)` when not already present, and writes the merged list. The read-merge-write runs inside a per-path lock, and the handler re-reads the page and errors if the edge is absent.
  - `wm_page.update` keeps **replace** semantics for `relates_to`: the caller supplies the complete list it wants.

  ## Rationale

  - The two operations express different intents. `link` is "add this edge"; `update` is "set these fields". Making both replace forced `link` callers to reconstruct the whole edge list, which is exactly what the handler failed to do.
  - Merge in the service layer (inside the lock), not the handler: the first attempt merged in the handler but read outside the lock, so parallel links still lost edges. Correctness required the read and the write to share one critical section.
  - Keeping `update` as replace preserves the existing contract and the documented `wm_page.update` payload, so no caller behaviour changes there.

  ## Consequences

  - `link` is idempotent for an existing `(type, target)` pair and safe to call repeatedly.
  - Any future append-only frontmatter field should follow the same split: a dedicated merge operation, not a widening of `update`.
  - `wm_page.update` callers must still send the full desired `relates_to` list; passing a partial list will drop edges.

  ## Related

  - @wiki/tasks/wmpagelink-concurrency-race-and-relates-to-clobber
  - @wiki/specs/wmpagelink-merge-and-write-serialization
  - @wiki/patterns/atomic-file-writes-unique-temp-and-path-lock
  - @wiki/rules/durable-write-or-error
questions:
  - id: outcome
    type: choice
    instructions: What is the recorded outcome of this decision?
    options:
    - adopted
    - rejected
    - deferred
    - superseded
    - abandoned
  - id: reversibility
    type: noul
    instructions: The decision can be reversed cheaply without data migration or cross-module breakage.
  - id: confidence
    type: score
    instructions: How strong is the recorded justification for the selected outcome?
    levels:
    - low
    - medium
    - high
  - id: impact
    type: choice
    instructions: How wide is the blast radius of this decision?
    options:
    - local
    - component
    - system
    - project-wide
answers: {}
