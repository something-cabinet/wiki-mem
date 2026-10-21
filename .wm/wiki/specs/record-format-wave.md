---
title: Record Format Wave — block-safe YAML, writer-backed updates, memory id
type: spec
id: "wiki:specs:record-format-wave"
status: approved
tags: [record-format, yaml, tool-reliability, spec]
acceptance_criteria:
  - checked: false
    text: Spec reviewed and its requirements are implemented
---

schema_version: 1
state: |-
  ## Goal

  Finish the record-format migration without re-doing what is already correct. Source-level tests prove the create paths
  (wm_page, wm_task, wm_memory, wm_doc) already emit record envelopes and apply extra_frontmatter. The remaining real
  defects were in the YAML block helpers, the update path's ad-hoc tag/AC serialization, memory id handling, and
  update durability under concurrency.

  ## Verified current state (2026-10-21)

  Passing source tests (the matching tasks are now closed):
  - `page_create_applies_extra_frontmatter_on_creation`
  - `task_create_persists_priority_and_acceptance_criteria`
  - `memory_add_emits_record_body`
  - `page_update_still_applies_extra_frontmatter`
  - `task_create_acceptance_criteria_round_trips_yaml_hostile_text`
  - `doc_update_persists_tags_and_preserves_type`

  Defects fixed in this wave:
  - `remove_yaml_block`, `set_yaml_value_field`, `set_yaml_block` consumed only indented lines, leaving orphaned
    column-0 list items and blank-line tails.
  - `page_update_builder_service.rs` wrote tags/ACs with `format!`, bypassing quoting/block shape.
  - `wm_memory.add` ignored an explicit id and always derived the title slug.
  - `wm_page.update` saved the version before the page write with no serialization, so concurrent updates could
    surface `IO_ERROR`.

  Still open (not in this wave):
  - Per-type schema enforcement (required keys, mandatory ACs) needs a decision matrix.
  - `unify-yaml-frontmatter-serializers` still lacks the CI grep guard required by its AC.
  - `wmdoccreate-writes-flat-prose` create half is fixed by the shared builder, but the validator rejection half is not
    implemented.

  ## Implementation (landed)

  - **YAML core**: `is_block_continuation` now accepts column-0 `-`/`*` list items (via `is_bare_list_item` and
    `LIST_ITEM_MARKERS`) and blank lines; all three block consumers use it. 9 new tests.
  - **Update path**: `apply_tags`/`apply_acceptance_criteria` call `set_yaml_value_field`; hostile tags/ACs round-trip
    verbatim.
  - **Memory id**: `WmMemoryAction::Add` takes `id: Option<String>`; `memory_slug_from_id` honors
    `wiki:memory:<slug>`/bare slug and rejects a mismatch with `INVALID_PARAMS`.
  - **Update durability**: a poison-recovering process-wide `PAGE_MUTATION_LOCK` serializes page mutations;
    `save_doc_version` runs after a successful write and is best-effort (warning only). Two-thread regression passes.
  - Note: the process-wide lock serializes updates to different pages too; acceptable for the current one-project,
    single-user process. Key it per page id if multi-page write concurrency becomes a measured bottleneck.

  ## Verification (re-run by the manager)

  - `cargo test -p wm-engine`: 122 lib + 3 integration.
  - `cargo test -p wm-core --lib`: 227.
  - `cargo test -p wm-tools --test mcp_test`: 119 (includes the new memory-id and concurrent-update tests).
  - `cargo clippy -p wm-engine -p wm-core -p wm-tools -- -D warnings`: clean.
  - rustfmt `--config skip_children=true --check` on all touched files: clean.
  - `cargo build -p wm-cli`: success.

  ## Out of scope

  - Rebuilding/installing the npm MCP binary (the running process is still 0.5.2; source verification is authoritative
    for this wave).
  - Per-type schema enforcement and the serializer CI guard.
questions:
  - id: kind
    type: choice
    instructions: What kind of spec is this?
    options:
    - feature
    - system
    - doc
    - migration
    - experiment
  - id: scope
    type: choice
    instructions: How wide is the scope of this spec?
    options:
    - local
    - component
    - system
    - project-wide
  - id: status_class
    type: choice
    instructions: What lifecycle class is this spec in?
    options:
    - draft
    - reviewed
    - approved
    - superseded
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers:
  kind: migration
  scope: component
  status_class: approved
  needs_tasks: true