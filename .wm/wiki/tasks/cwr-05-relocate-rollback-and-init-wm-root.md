---
title: CWR-05 relocate/rollback verbs + wm init --wm-root
type: task
id: "wiki:tasks:cwr-05-relocate-rollback-and-init-wm-root"
status: todo
tags: [from-spec, spec:configurable-wm-root, cli, config]
acceptance_criteria:
  - text: relocate ordering is copy to verify to stub to remove, tested with an induced I/O failure leaving the original intact and no stub written
  - text: '`wm config relocate --to .` restores default resolution and removes the stub'
  - text: '`wm init --wm-root` works non-interactively and the family-branch recipe is documented'
priority: high
---

schema_version: 1
state: |-
  Implements D10 with AC-9 of @wiki/specs/configurable-wm-root

  - `wm config relocate --to <path>`: refuse a non-empty target → copy → verify (file count plus size/hash spot-check) → write the stub → **only then** remove the original. The stub is not declared until the payload is verified, so an induced failure leaves the original intact and no stub written (`durable-write-or-error`).
  - Rollback: `wm config relocate --to .` — symmetric move back plus stub removal. The registry approval entry persists harmlessly.
  - `wm init` gains non-interactive `--wm-root`.
  - No copy-and-keep-both variant: two live state dirs is precisely the split-state failure this design exists to prevent.
  - Document the family-branch recipe: initialise once, stub the siblings at the shared target.
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