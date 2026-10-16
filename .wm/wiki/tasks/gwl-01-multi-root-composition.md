---
title: GWL-01 Multi-root composition — one engine, one index, two roots
type: task
id: "wiki:tasks:gwl-01-multi-root-composition"
status: cancelled
tags: [from-spec, spec:global-wiki-layer, engine, search]
acceptance_criteria:
  - text: A global concept page is found by wm_search, appears in the graph, and a relates_to edge from a project page resolves in both directions
  - text: 'Exactly one EngineStateMediator; zero new dependencies; no #[allow(dead_code)]; warning-free build and clippy'
  - text: wm_root resolving to $HOME/.wm composes a single root with scope global plus a warning, producing no doubled nodes
priority: high
---

schema_version: 1
state: |-
  Implements D1 and D8 with AC-1, AC-6 and AC-11 of @wiki/specs/global-wiki-layer

  - `build_graph_from_wiki` (`apps/wm-core/src/graph/mod.rs:51`) and `build_sections_from_wiki` (`freshness_service.rs:20`) take multiple roots and merge; the call site at `freshness_service.rs:14-20` composes both.
  - `is_wiki_page` (`main_engine_factory.rs:97`) tests containment against any registered root.
  - Engine state holds two roots; exactly one `EngineStateMediator` remains. No second engine, no second index.
  - Dedupe case: `wm_root` resolving to `$HOME/.wm` composes **one** root with scope `global` and warns — never index the same canonical path twice.
  - Register the global root with the watcher (`main_engine_factory.rs:263`); if registration fails (network home, platform limits) degrade with a visible warning, never silently stale.
  - Coordinate with @wiki/specs/configurable-wm-root: it re-points the project slot, so `state_root()` and scope resolution share a surface and must land together.
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