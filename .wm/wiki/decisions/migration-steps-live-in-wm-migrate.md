---
title: 'Decision: migration steps live in the wm-migrate package'
type: decision
id: "wiki:decisions:migration-steps-live-in-wm-migrate"
status: draft
relates_to:
  - {type: implements, target: wiki:tasks:add-wm-cli-migrate-migrate-the-whole-wiki-with-versioned-records}
---

schema_version: 1
state: |-
  ## Context

  wm-cli migrate had to run over every page and legacy memory entry, and the file-system record migration (1,095 lines) lived in wm-core. wm-core sat 15 lines under its 13000-line ceiling, so new migration steps could not be added there.

  ## Decision

  The file-system-only migration steps live in a new package, packages/wm-migrate, which depends only on wm-engine. Steps that need the engine, such as the legacy memory import, stay in wm-core and wm-cli. wm-core depends on wm-migrate for build_record_body, which its create paths use.

  Version history is not written inside wm-migrate. The applier calls an observer with the old and new text just before each write, and wm-cli records one content version per rewritten page through VersionStore (task history for tasks, doc history for everything else). The memory import records the first version of each page it creates.

  ## Rationale

  - A new package is the structure the size gate asks for; raising the ceiling would hide the problem.
  - wm-migrate cannot depend on wm-core without a cycle, so the observer keeps history writing in the layer that owns VersionStore.
  - Every rewrite is reversible from .wm/versions because the old text is stored.

  ## Consequences

  - wm-core dropped to 11,949 lines, 1,051 under the ceiling.
  - A record version bump adds one VersionStep to wm-migrate; old projects upgrade in order.
  - migrate-memory and page migrate-records no longer exist as commands.
  - The CI integration job now runs memory_import_test and migrate_command_test.
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