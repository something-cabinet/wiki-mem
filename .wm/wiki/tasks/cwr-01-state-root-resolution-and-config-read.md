---
title: CWR-01 state_root() accessor + two-tier config read
type: task
id: "wiki:tasks:cwr-01-state-root-resolution-and-config-read"
status: todo
tags: [from-spec, spec:configurable-wm-root, config, paths]
acceptance_criteria:
  - text: detect_project_root() behaviour is unchanged when no wm_root is present (existing suite passes unmodified)
  - text: 'A stub { "wm_root": "..." } causes the engine to compose the resolved root; search, graph, page CRUD and audit sink all operate on it'
  - text: Six distinct startup errors (dangling target, split state, non-rooted, filesystem-root, home-exact, wm_root in resolved config) each naming its constraint, with zero silent fallbacks
priority: high
---

schema_version: 1
state: |-
  Implements the state-root resolution half of @wiki/specs/configurable-wm-root (D2, D3, D4, D5; AC-1, AC-2, AC-3).

  Scope:
  - `detect_project_root()` stays untouched (`apps/wm-core/src/config/models/git_tracking_model.rs:56-74`) — `<root>/.wm/config.json` remains the discovery anchor, including the `WM_PROJECT` override and the `d7f99f` symlink handling.
  - New single accessor `state_root()` resolves the effective state directory. Migrate every `root.join(WM_DIR)` consumer: `audit_sink.rs:53`, `init_service.rs:165`, `freshness_service.rs:14`, `main_engine_factory.rs:97` and `:263`, `doc.rs:49`, `template/mod.rs:12`. The migration is compiler-led.
  - Two-tier read: anchor config first; if it carries `wm_root`, the config at `<wm_root>/config.json` is authoritative for all other keys.
  - `wm_root` present in the *resolved* config is a startup error (kills redirect cycles).
  - Six distinct startup errors, none falling back silently: dangling target, split state, non-rooted value, filesystem-root value, home-exact value, and `wm_root` in the resolved config.

  Constraint: silent fallback to the default `.wm` splits state, which is the worst available outcome.
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