---
title: "Fix three pre-existing clippy warnings (pre-existing on master)"
id: "wiki:tasks:fix-pre-existing-clippy-warnings"
type: task
status: todo
priority: medium
tags: [chore, lint, clippy, cleanup]
acceptance_criteria:
  - text: "apps/wm-core/src/graph/mod.rs:613 — replace .get(..).is_none() with !contains_key(..)"
  - text: "apps/wm-core/src/search/query.rs:516 — useless vec! becomes an array (results is sorted later, so mut stays)"
  - text: "apps/wm-core/tests/helpers/inproc.rs:46 — call_err is used by security_test.rs/e2e_mcp.rs but unused in other test binaries; resolved without #[allow(dead_code)]"
  - text: "cargo clippy --workspace --all-targets emits zero warnings"
---

schema_version: 1
state: |-
  Found during EXTRACT-01 verification (2026-10-16). `cargo clippy --workspace --all-targets` reports four warnings, none introduced by the extraction - all three sites are byte-identical to master (`graph/mod.rs:613`, `search/query.rs:516`, `tests/helpers/inproc.rs:46`). CI does not run clippy (build, tests, audit and a wm-cli file-size budget only), so these are latent against the `no-warnings` rule.

  The `call_err` case is the only non-trivial one: the helper is used by `security_test.rs` and `e2e_mcp.rs` but unused in test binaries that include the shared helpers module, so it cannot simply be deleted and `#[expect(dead_code)]` would error where the lint does not fire. Fix without `#[allow(dead_code)]`.
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
