---
title: Unify the YAML frontmatter serializers into one conformant writer
id: "wiki:tasks:unify-yaml-frontmatter-serializers"
type: task
status: todo
priority: high
tags: [from-spec, yaml, frontmatter, serializer, migration]
acceptance_criteria:
  - text: "Exactly one YAML frontmatter serializer exists workspace-wide, asserted by grep count in CI"
  - text: "Pinned round-trip tests cover all six byte-shapes, written RED before the unification"
  - text: "The five timer-recovery task pages and any block-sequence users migrate with no data loss"
  - text: "cargo build, test and clippy --workspace are green"
---

schema_version: 1
state: |-
  Unify the three workspace YAML frontmatter serializers into one conformant writer, extending `FrontmatterValue` to express block sequences and migrating the affected pages.

  Follow-up from @wiki/specs/wm-core-layer-extraction D-10. Spec: @wiki/specs/yaml-frontmatter-serializer-unification.

  Scope: `frontmatter_to_yaml` (live, via `page/services/timer_recovery_service.rs`), the `build_frontmatter` field builder, and the third copy. `graph/export.rs` stays out of scope.

  Blocked on the owner's byte-output decision - the open question in the spec. Do not resolve it in implementation.
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
