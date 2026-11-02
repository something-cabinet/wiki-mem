---
title: wm-cli global --dry-run flag for write commands
type: spec
id: "wiki:specs:wm-cli-global-dry-run-flag-for-write-commands"
status: draft
tags: [cli, dry-run, safety, spec]
acceptance_criteria:
  - text: wm-cli accepts --dry-run before or after any subcommand
  - text: With --dry-run, every write command prints what it would change and writes nothing to disk
  - text: A test per write command proves the file system is unchanged after a dry run
  - text: Read-only commands accept the flag and ignore it
  - text: The existing health audit --dry-run and the migrate-records --apply plan behaviour are folded into the global flag
---

schema_version: 1
state: |-
  ## Goal

  A global --dry-run flag that makes any wm-cli write command report its changes and write nothing.

  ## Verified current state

  - There is no global flag. Dry run exists only in two places: health audit has --dry-run and --fix (it plans unless --fix is given), and page migrate-records plans unless --apply is given. The two use different words and opposite defaults.
  - Commands that write: init, setup, agents --sync, index rebuild, index embed, index code, page create, page delete, page update, page link, page unlink, source remove, time start, time stop, time add, lint fix, model download, model remove, config set, health audit --fix, and the migrate command from the migrate spec.
  - Read-only commands: search, graph (export writes only with --out), page get and list, task board, status, validate, lint check, config get and list, log, version.

  ## Options

  - A. A clap global argument, with each write command receiving a dry-run value and branching before it touches the disk. One flag, explicit per command. Recommended.
  - B. Run every write against a copy of the project and print the diff. Needs no per-command code but copies a large .wm directory and runs the index builds twice.
  - C. A context object that every file write goes through, so the write layer itself skips writing. Cleanest long term, but the writes are spread over wm-core, wm-tools and wm-cli today.

  ## Open questions

  - Option A, B or C.
  - The MCP tools: does wm_page and wm_task get a dry_run argument too, or only the CLI.
  - health audit --fix and page migrate-records --apply: remove the old flags or keep them as aliases for one release.
  - Exit code on a dry run that finds problems: 0, or non-zero as a check mode.

  ## Out of scope

  The migrate command itself, which only needs to honour the flag once it exists.

  ## Verification

  - A snapshot of the project tree before and after a dry run of each write command is identical.
  - A real run of the same command afterwards produces the changes the dry run reported.
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
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers:
  kind: feature
  scope: project-wide
  needs_tasks: true
