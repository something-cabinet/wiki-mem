---
title: Windows release import fix
type: spec
id: "wiki:specs:windows-release-import-fix"
status: approved
tags: [ci, windows, release, spec]
---

schema_version: 1
state: |-
  ## Goal

  Fix the Windows-only release build failure without changing Unix lock-holder diagnostics.

  ## Verified current state

  - Release run 39890622851: check, tests, Linux, and macOS publish jobs pass; only
    `publish (x86_64-pc-windows-msvc)` fails.
  - Error: `unused import: std::process::Command` at
    `packages/wm-code-intel/src/helpers/lock_error_helper.rs:2:5`, promoted by `-D warnings`.
  - `Command` is used only by `lsof_pid` and `process_name`, both `#[cfg(unix)]`; the non-Unix stubs
    return `None`.

  ## Change

  Gate the import with `#[cfg(unix)]`. No behavior change on any platform.

  ## Verification

  - `cargo clippy -p wm-code-intel -- -D warnings` (Unix) and the four lock-helper tests.
  - Master CI green.
  - Re-tag v0.6.1 at the fix commit (no GitHub Release or published artifact exists) and confirm all four publish
    targets build.

  ## References

  - @wiki/tasks/windows-release-build-fails-on-a-unix-only-command-import
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
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers:
  kind: system
  scope: local
  status_class: approved
  needs_tasks: true
