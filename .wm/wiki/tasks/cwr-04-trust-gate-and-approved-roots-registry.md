---
title: CWR-04 Trust gate + approved-roots registry
type: task
id: "wiki:tasks:cwr-04-trust-gate-and-approved-roots-registry"
status: todo
tags: [from-spec, spec:configurable-wm-root, security]
acceptance_criteria:
  - text: An unapproved target errors naming `wm config accept-root`; after approval it succeeds, is registry-recorded and audit-logged
  - text: Canonicalisation precedes the gate, so two spellings of one target share a single approval
  - text: Every redirect resolution is audit-logged at session start with target and registry hit/miss
priority: high
---

schema_version: 1
state: |-
  Implements D6 with AC-4 of @wiki/specs/configurable-wm-root

  - An unrecognised `wm_root` target is a **startup error** until approved once via `wm config accept-root <path>`.
  - Without the gate, a cloned repository could carry a stub pointing at any writable path and every wiki write would follow it.
  - Registry: `~/.wm/projects.json` (introduced by @wiki/specs/global-wiki-layer) gains an approved-roots list of canonical expanded paths plus timestamps. Canonicalisation precedes the gate, so two spellings of one target share one approval.
  - Log every redirect resolution at session start (target, registry hit/miss) so "why is my wiki in /unexpected/place" is always answerable.
  - Post-gate blast radius of a malicious or mistaken stub must be exactly "nothing not explicitly approved".
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