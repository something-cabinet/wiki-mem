---
title: GWL-05 wm-cli global audit + projects registry + opt-out
type: task
id: "wiki:tasks:gwl-05-global-audit-registry-and-opt-out"
status: cancelled
tags: [from-spec, spec:global-wiki-layer, cli, config]
acceptance_criteria:
  - text: $HOME/.wm/projects.json is advisory only, never read on the write-critical path, and tolerant of corrupt or missing entries
  - text: '`wm-cli global audit` against the five real projects reports at least two same-identity-different-SHA pairs and a non-empty name-cluster report'
  - text: ignore_global_wiki yields project-only composition, verified by test
priority: high
---

schema_version: 1
state: |-
  Implements D7 and D11 with AC-11 and AC-12 of @wiki/specs/global-wiki-layer

  - `$HOME/.wm/projects.json`: a best-effort registry appended by `wm init`/`wm setup` (deduped by path, tolerant of corrupt or missing entries). Advisory only — never read on the write-critical path, never an error source. A JSON file, not a database.
  - `wm-cli global audit` reports: same-identity-different-SHA across scopes; the shadow report; name-similarity clusters; and any non-shareable pages ignored in the global root.
  - Run against the five real projects under `/Users/nhkhanh/projects` it must reproduce the evidence table: at least two same-identity-different-SHA pairs and a non-empty name-cluster report. Use that as the integration fixture.
  - Opt-out key `ignore_global_wiki` yields project-only composition.
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