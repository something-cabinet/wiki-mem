---
title: Make the embedder injectable in EngineState for tests
type: task
id: "wiki:tasks:make-the-embedder-injectable-in-enginestate-for-tests"
status: done
priority: medium
tags: [testing, embeddings, engine]
spec: wiki:specs:injectable-embedder-for-engine-state-tests
acceptance_criteria:
  - text: EngineState can be built with a caller-supplied embedder and vector store
    checked: true
  - text: EngineState::new keeps its signature and behaviour
    checked: true
  - text: A wm-tools test proves a successful wm_index_rebuild writes vectors that VectorStore::load_from_disk reads back
    checked: true
  - text: A wm-tools test proves a store that cannot persist fails wm_index_rebuild and wm_index_embed before the embedder is called
    checked: true
implementation_notes: 'Option A: EngineState::with_embedder added in apps/wm-core/src/engine/engine_state_mediator.rs; new() builds the same embedder and store and delegates to it. Tests in packages/wm-tools/tests/index_persist_test.rs (4) failed to compile before the constructor existed. Mutation checks: making persist_vectors a no-op fails the read-back test; making the pre-embed guard a no-op fails both fail-before-embedding tests.'
---

schema_version: 1
state: |-
  IndexState takes its embedder only from init_embedder, so no test can run wm_index_rebuild or wm_index_embed with a loaded embedder. The persist task has a criterion open because of it. Option A in the spec: EngineState::with_embedder. Waiting for the owner to approve the spec and pick an option.
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