---
title: "timer_recovery writes duplicate frontmatter keys (time_started, status) — decide overwrite vs skip"
id: "wiki:tasks:timer-recovery-frontmatter-duplicate-keys"
type: task
status: todo
priority: medium
tags: [bug, frontmatter, timer-recovery, parser, wm-core]
acceptance_criteria:
  - text: "timer_recovery_service.rs no longer appends time_started: when the serializer already emitted it (no duplicate key)"
  - text: "The status overwrite-vs-skip semantics is stated explicitly and covered by a test"
  - text: "No duplicate top-level YAML key is produced by any recovery write"
---

schema_version: 1
state: |-
  Surfaced during EXTRACT-02 review (2026-10-16); pre-existing, out of that task's scope.

  `apps/wm-core/src/page/services/timer_recovery_service.rs` recomposes task frontmatter on recovery. Two duplicate-key warts:
  - Line ~61 still appends `time_started:` even though the serializer already emitted it (the old code did the same).
  - The new guard for `status` (append `status: done` only when absent) changed a latent semantic: a recovered orphan task that already carries `status:` no longer gets `status: done`. Whether recovery should overwrite or skip is a deliberate decision, not an accident - state it and test it.

  Related to the deferred serializer unification (D-10) and to @wiki/specs/wm-core-layer-extraction.
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
