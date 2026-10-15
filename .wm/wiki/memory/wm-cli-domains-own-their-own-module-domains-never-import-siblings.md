---
title: wm-cli domains own their own module; domains never import siblings
type: memory
tags: [architecture, wm-cli, domain-driven, module-layout]
status: active
---

schema_version: 1
state: |-
  `apps/wm-cli` is organised domain-driven: each domain (command) is a flat feature directory at the crate root with its own `mod.rs` and role-suffixed files (`<domain>_service.rs`, `<domain>_args_model.rs`, `<domain>_renderer.rs`, `<domain>_helper.rs`). This matches the repo` s established shape, where role directories sit INSIDE feature directories (`wm-core/src/page/{helpers,services}`, `wm-core/src/skill/{models,services,helpers}`), and the flat precedent `wm-core/src/page/services/record_migration/`.
  A domain owns its own model and behaviour together. The enforceable consequences:
  1. A domain must NOT import a sibling domain. Cross-domain needs go to `shared/` (the shared kernel: `shared/traits/command.rs`, `shared/models/{cli_model,commands_model}.rs` for the crate-wide clap surface, `shared/services/*` for engine/agent-sync/platform-MCP, `shared/helpers/*`).
  2. Only the router is allowed to know every domain; it is the composition root.
  3. A domain is `models/` + `service` + `renderer` + `helper` in one directory, so a domain never has two homes.
  Verified at the time of the wm-cli restructure: no command service imported a sibling command service; dependencies pointed at `shared/` and `wm-core` only.
  Superseded layout note: the first version of the restructure used a root `services/<domain>/` role directory with the domains nested inside it, which inverted the repo shape and split each domain across `services/<domain>/` and `shared/models/<domain>_args_model.rs`. Reviewer rejected it for exactly that two-homes reason. Do not reintroduce a root role directory that contains domain directories.
  Refs: @wiki/decisions/wm-cli-main-rs-restructure-map, @wiki/tasks/controlled-rewrite-of-appswm-clisrcmainrs--one-architecture-decision-per-review
questions:
  - id: layer
    type: choice
    instructions: Which memory layer does this entry belong to?
    options:
    - project
    - global
    - session
  - id: store_or_skip
    type: noul
    instructions: This entry is worth storing as durable memory.
  - id: dedup_action
    type: choice
    instructions: How should this entry relate to existing memory?
    options:
    - new
    - merge
    - supersede
    - skip
  - id: confidence
    type: score
    instructions: How confident is the recorded knowledge?
    levels:
    - low
    - medium
    - high
answers: {}
