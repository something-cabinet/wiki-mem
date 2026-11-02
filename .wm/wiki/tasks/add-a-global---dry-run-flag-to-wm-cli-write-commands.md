---
title: Add a global --dry-run flag to wm-cli write commands
type: task
id: "wiki:tasks:add-a-global---dry-run-flag-to-wm-cli-write-commands"
status: todo
priority: medium
tags: [cli, dry-run, safety]
spec: wiki:specs:wm-cli-global-dry-run-flag-for-write-commands
acceptance_criteria:
  - text: wm-cli accepts --dry-run before or after any subcommand
  - text: With --dry-run, every write command prints what it would change and writes nothing to disk
  - text: A test per write command proves the file system is unchanged after a dry run
  - text: Read-only commands accept the flag and ignore it
  - text: The existing health audit --dry-run and the migrate-records --apply plan behaviour are folded into the global flag
---

schema_version: 1
state: |-
  Owner decision while specifying wm-cli migrate: --dry-run should be global for all write commands. Today only health audit (--dry-run, applies with --fix) and page migrate-records (plans unless --apply) have anything like it, with opposite defaults. The spec lists the 21 write commands and three implementation options. Blocked on the owner's choice of option and answers to the open questions.
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
