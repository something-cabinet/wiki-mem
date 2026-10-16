---
title: 'Decision: Compose two source roots into one engine'
type: decision
id: "wiki:decisions:global-wiki-multi-root-composition"
status: superseded
tags: [decision, architecture, global, search]
relates_to:
  - {type: implements, target: wiki:specs:global-wiki-layer}
---

schema_version: 1
state: |-
  ## Context

  The engine is rooted in one directory: `project_root: RwLock<PathBuf>` (`apps/wm-core/src/engine/engine_state_mediator.rs:35`), with the graph (`build_graph_from_wiki`, `graph/mod.rs:51`), BM25, the section index and `wm_initial` counts all deriving from `project_root/.wm/wiki/`. Knowledge recurring across the owner's five projects was therefore re-authored per project and measurably diverged — four names for "no comments in code", four for spec-first, and `no-comments-in-code` at 498 B in one repo against 2149 B in another under the same name.

  ## Decision

  Compose **exactly two** source roots into one engine and one index: the project wiki and `$HOME/.wm/wiki`. Page identity is `(type, slug)`; the project copy wins and the global copy is shadowed and reported.

  ## Rationale

  Two alternatives were rejected. **Shadow-sync** (`synced_from: global@<hash>` plus a divergence lint) would have institutionalised the measured drift — copies that diverge, detected only by additional machinery whose only job is to compensate. **Read-time overlay** (the superseded `global-rules-cascade`) is valid for rules, which are read in full at session start, but concepts, patterns, howtos and decisions are *searched*, so an overlay leaves them invisible to `wm_search`, the graph and `wm_initial`.

  Multi-root costs a signature change on two already-`fn(&Path)`-shaped builders plus a merge step, needs no new dependencies, and keeps one `EngineStateMediator`.

  ## Consequences

  `wiki:core:ARCHITECTURE` gains an amendment recording that scope is provenance and that there are exactly two scopes. Global pages become searchable and graphable. The watcher must cover the global root or degrade with a visible warning. A corrupt `$HOME/.wm` affects every project's effective wiki, and concurrent global writes are last-writer-wins.

  ## Related

  - @wiki/specs/global-wiki-layer

  ## Superseded (2026-10-15)

  Reversed by owner decision: the global wiki layer is removed, manual per-project duplication accepted, and the multi-root composition described here is not being built. Retained for the record. Note the sibling decision wiki:decisions:scope-is-provenance-not-page-data is **not** superseded - it remains valid and is what justifies deleting MemoryLayer::Global in wiki:specs:remove-global-memory-layer.
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