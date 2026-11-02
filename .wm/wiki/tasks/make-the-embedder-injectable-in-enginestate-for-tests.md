---
title: Make the embedder injectable in EngineState for tests
type: task
id: "wiki:tasks:make-the-embedder-injectable-in-enginestate-for-tests"
status: todo
priority: medium
tags: [testing, embeddings, engine]
spec: wiki:specs:injectable-embedder-for-engine-state-tests
acceptance_criteria:
  - text: EngineState can be built with a caller-supplied embedder and vector store
  - text: EngineState::new keeps its signature and behaviour
  - text: A wm-tools test proves a successful wm_index_rebuild writes vectors that VectorStore::load_from_disk reads back
  - text: A wm-tools test proves a store that cannot persist fails wm_index_rebuild and wm_index_embed before the embedder is called
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
