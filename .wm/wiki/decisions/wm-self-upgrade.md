---
title: "Decision: Binary Self-Deployment via wm upgrade"
id: wiki:decisions:wm-self-upgrade
type: decision
status: archived
tags: [decision, deployment, self-upgrade, path]
implementation_notes: 'SUPERSEDED by @wiki/specs/remove-self-install-flow (approved 2026-07-31). Self-deployment via wm upgrade / ~/.wm/bin is removed; cargo-npm distribution replaces it.'
relates_to:
  - {type: superseded_by, target: wiki:specs:remove-self-install-flow}
decision:
---

schema_version: 1
state: |-
  ## Context

  Binary self-deployment via `wm upgrade` was superseded by the cargo-npm distribution channel (see @wiki/specs/remove-self-install-flow). The self-install flow was removed; this decision is archived.
questions:
  - id: outcome
    type: choice
    instructions: What is the recorded outcome of this decision?
    options: [adopted, rejected, deferred, superseded, abandoned]
  - id: reversibility
    type: noul
    instructions: The decision can be reversed cheaply without data migration or cross-module breakage.
  - id: confidence
    type: score
    instructions: How strong is the recorded justification for the selected outcome?
    levels: [low, medium, high]
  - id: impact
    type: choice
    instructions: How wide is the blast radius of this decision?
    options: [local, component, system, project-wide]
answers: {}
