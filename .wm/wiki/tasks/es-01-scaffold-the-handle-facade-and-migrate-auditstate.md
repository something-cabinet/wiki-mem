---
title: ES-01 Scaffold the handle facade and migrate AuditState
type: task
id: "wiki:tasks:es-01-scaffold-the-handle-facade-and-migrate-auditstate"
status: done
priority: high
acceptance_criteria:
  - text: "EngineState is a struct of per-domain handle fields; AuditState exists and owns audit_sender, audit_drops and started_at"
  - text: "A unit test proves AuditState owns its three fields and exposes them only through methods, with no sibling handle reading them directly"
  - text: "cargo build, test and clippy --workspace are green with no new compiler or clippy warnings"
  - text: "The change is independently revertible and touches no other domain's fields"
tags: [from-spec, spec:engine-state-decomposition, refactor, wm-core, engine]
implementation_notes: |-
  Implemented on 2026-10-16.

  AuditState added at apps/wm-core/src/engine/state/models/audit_state_model.rs, owning audit_sender,
  audit_drops and started_at as private fields with new/emit/dropped_count/started_at. EngineState lost the
  three flat fields and gained pub audit: AuditState; emit_audit keeps its exact signature and delegates, so
  the only external consumer (timer_recovery_service.rs:79) compiles unchanged.

  Verified: cargo test -p wm-core --lib = 192 passed / 0 failed (4 new tests, 188 before);
  cargo check --workspace --all-targets green. Clippy reports 2 warnings, both pre-existing and in files this
  change does not touch (graph/mod.rs:613, search/query.rs:516).

  Pre-existing, not caused here: apps/wm-core/tests/fixtures/cli_golden/version.txt expects 'wm 0.5.2' while
  apps/wm-cli/Cargo.toml declares 0.6.0 at HEAD, so cli_characterization_test fails on any build and aborts
  the remaining test binaries under a plain cargo test --workspace.

  Finding for a follow-up ruling: EngineState.started_at was write-only before this change - no read exists
  anywhere in the workspace. It is preserved on AuditState per D-5 and reachable only via the accessor.
---

schema_version: 1
state: |-
  First step of @wiki/specs/engine-state-decomposition (D-7 leaf-first ramp). Introduce the state-handle
  scaffold inside `EngineState` and migrate the first and lowest-risk domain: `AuditState`, owning
  `audit_sender`, `audit_drops` and `started_at`.

  The scaffold is what makes the remaining six migrations cheap. `EngineState` becomes a struct of handle
  fields, constructed at the single construction site in `engine_state_mediator.rs`, so each later domain
  becomes a field move plus a method surface rather than a rewrite. Keep the facade shape of D-1: call sites
  keep `engine.<handle>.<method>()` access and no consumer type changes.

  Scope is deliberately one domain. Do not migrate any other domain in this task.
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