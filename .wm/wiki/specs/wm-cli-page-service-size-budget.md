---
title: wm-cli page_service size budget — extract the payload helper
type: spec
id: "wiki:specs:wm-cli-page-service-size-budget"
status: approved
tags: [ci, wm-cli, structure, spec]
acceptance_criteria:
  - checked: false
    text: Spec reviewed and its requirements are implemented
---

schema_version: 1
state: |-
  ## Goal

  Restore a green CI `check` job by bringing `apps/wm-cli/src/page/page_service.rs` under the enforced 300-line
  budget (currently 332).

  ## Verified current state

  - CI run 39353845207 (first failure, docs commit 2026-10-20T01:30Z) fails only at
    `Enforce wm-cli file-size budget` with `apps/wm-cli/src/page/page_service.rs: 332 lines > 300`.
  - Every later run fails for the same reason; no other check step failed.
  - `03f4e4f refactor(wm-core): extract LifecycleState; make page update reject bare prose` added the update
    payload logic that pushed the file over budget.
  - No other wm-cli file exceeds 300 lines (next: health_service.rs at 297).

  ## Design

  Extract the update-payload/input cluster into `apps/wm-cli/src/page/page_payload_helper.rs`:

  - `read_stdin`, `read_update_payload`, `apply_content_file`, `read_content_file`,
    `parse_update_payload`, `attach_update_identity`
  - constants `STDIN_READ_ERROR`, `STDIN_JSON_ERROR`, `STDIN_OBJECT_ERROR`, `CONTENT_FIELD`,
    `CONTENT_SOURCE_CONFLICT`, `CONTENT_FILE_READ_ERROR`, `ACTION_UPDATE`
  - the three content-file tests

  `page_service.rs` keeps the command dispatch, create/get/list/delete/link/unlink/migrate handlers, and imports the
  helper functions it still needs (`read_stdin`, `read_update_payload`, `attach_update_identity`).

  ## Verification

  - `wc -l apps/wm-cli/src/page/page_service.rs` <= 300.
  - `cargo test -p wm-cli` green.
  - `cargo clippy -p wm-cli -- -D warnings` clean; rustfmt clean on both files.
  - Re-run the CI shell check locally:
    `find apps/wm-cli/src -name '*.rs' ...` reports no violation.

  ## Out of scope

  - Moving the crate to the role-based subdirectory layout.
  - Any behavior change to page update/create.
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
  kind: doc
  scope: local
  status_class: approved
  needs_tasks: true