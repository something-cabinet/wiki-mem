---
title: Injectable embedder for EngineState tests
type: spec
id: "wiki:specs:injectable-embedder-for-engine-state-tests"
status: draft
tags: [testing, embeddings, engine, spec]
acceptance_criteria:
  - text: EngineState can be built with a caller-supplied embedder and vector store
  - text: EngineState::new keeps its signature and behaviour
  - text: A wm-tools test proves a successful wm_index_rebuild writes vectors that VectorStore::load_from_disk reads back
  - text: A wm-tools test proves a store that cannot persist fails wm_index_rebuild and wm_index_embed before the embedder is called
---

schema_version: 1
state: |-
  ## Goal

  Let tests drive wm_index_rebuild and wm_index_embed with an embedder that is loaded, so the persistence guarantees are proven by a test and not by hand.

  ## Verified current state

  - EngineState::new calls init_embedder(config, root) and hands the result to IndexState::new. IndexState keeps the embedder in a private field with no setter.
  - Tests build EngineState through EngineState::new, so the embedder is always the no-op one unless an ONNX model sits under ~/.wm/models. Every wm_index_rebuild test therefore skips embedding.
  - wm-embed already exports MockEmbedder, a deterministic hash-based embedder with is_loaded true.
  - The persist task (index-rebuild-reports-success-when-vectors-cannot-persist) has one criterion open for this reason: a test showing a successful rebuild changes vectors.db.

  ## Options

  - A. Add EngineState::with_embedder(config, root, embedder, vector_store) and make new() call it with init_embedder. No new interior mutability, no test-only cfg. Recommended.
  - B. Add IndexState::replace_embedder behind an ArcSwap. Lets a test swap the embedder after construction, but adds runtime machinery to a field that never changes in production.
  - C. A cfg(test) or feature-gated constructor. Does not reach integration tests in other crates.

  ## Change (option A)

  - Split EngineState::new into new (unchanged signature) and with_embedder.
  - Add tests in packages/wm-tools using MockEmbedder and a CountingEmbedder wrapper:
    - rebuild with a persistable store writes vectors that load_from_disk returns;
    - rebuild and embed with a store whose state directory cannot be opened return an error and leave the call count at zero.

  ## Out of scope

  The ONNX embedder, model download, and the search paths.

  ## Verification

  - The two new tests fail first (no constructor), then pass.
  - cargo test --workspace stays green and clippy is clean on the touched crates.
questions:
  - id: kind
    type: choice
    instructions: What kind of spec is this?
    options:
    - feature
    - system
    - doc
    - migration
    - experiment
  - id: scope
    type: choice
    instructions: How wide is the scope of this spec?
    options:
    - local
    - component
    - system
    - project-wide
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers:
  kind: system
  scope: component
  needs_tasks: true
