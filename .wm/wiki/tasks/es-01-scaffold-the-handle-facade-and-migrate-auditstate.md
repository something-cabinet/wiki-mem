---
title: ES-01 Scaffold the handle facade and migrate AuditState
type: task
id: "wiki:tasks:es-01-scaffold-the-handle-facade-and-migrate-auditstate"
status: todo
priority: high
acceptance_criteria:
  - text: "EngineState is a struct of per-domain handle fields; AuditState exists and owns audit_sender, audit_drops and started_at"
  - text: "A unit test proves AuditState owns its three fields and exposes them only through methods, with no sibling handle reading them directly"
  - text: "cargo build, test and clippy --workspace are green with no new compiler or clippy warnings"
  - text: "The change is independently revertible and touches no other domain's fields"
tags: [from-spec, spec:engine-state-decomposition, refactor, wm-core, engine]
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
