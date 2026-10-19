---
title: ES-05 Migrate IndexState
type: task
id: "wiki:tasks:es-05-migrate-indexstate"
status: done
priority: high
acceptance_criteria:
  - text: "IndexState owns section_corpus, bm25_index, vector_registry, embedder and vector_store"
  - text: "ArcSwap is preserved for section_corpus and bm25_index; no new lock appears on the search read path (NFR-1)"
  - text: "Search results and index rebuild behaviour are unchanged (FR-3)"
  - text: "A unit test proves IndexState owns its state and does not reach into a sibling handle"
  - text: "cargo build, test and clippy --workspace are green with no new warnings"
tags: [from-spec, spec:engine-state-decomposition, refactor, wm-core, search]
implementation_notes: |-
  Implemented in 08475f4 (work completed by a specialist whose session was cancelled mid-run; the code survived
  and was verified afterwards).

  IndexState owns section_corpus, bm25_index, vector_registry, embedder and vector_store privately. Flat
  EngineState fields went 13 -> 8.

  NFR-1 verified structurally: the three hot fields remain bare ArcSwap, and a unit test asserts the handle body
  contains no Mutex and no RwLock.
---

schema_version: 1
state: |-
  Fifth step of @wiki/specs/engine-state-decomposition (D-7), and the first migration on the read-hot
  path. Migrate `IndexState`, owning `section_corpus`, `bm25_index`, `vector_registry`, `embedder` and
  `vector_store`.

  NFR-1 is the constraint: the `ArcSwap` snapshot pattern is preserved for `section_corpus` and `bm25_index`,
  and no new lock is introduced on the read path. `section_corpus` belongs here rather than with page content
  because `freshness_service` and `graph` write the corpus and the index together, per the D-5 ruling in the
  spec. Depends on ES-01.
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