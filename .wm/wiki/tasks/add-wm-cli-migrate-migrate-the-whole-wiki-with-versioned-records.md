---
title: 'Add wm-cli migrate: migrate the whole wiki, with versioned records'
type: task
id: "wiki:tasks:add-wm-cli-migrate-migrate-the-whole-wiki-with-versioned-records"
status: todo
priority: high
tags: [cli, migration, records]
spec: wiki:specs:wm-cli-migrate-upgrades-the-whole-wiki-with-versioned-records
acceptance_criteria:
  - checked: false
    text: wm-cli migrate migrates the entire wiki in order and is idempotent
  - checked: false
    text: migrate-memory and page migrate-records are gone from the CLI and live on as internal steps of migrate
  - checked: false
    text: It repairs pages whose first frontmatter block is a stub, merging the blocks so no field is lost
  - checked: false
    text: It converts legacy memory json and prose bodies to records
  - checked: false
    text: Every record carries a version; a record or page with none is version 0 and is upgraded step by step to the current version
  - checked: false
    text: It writes by default and writes nothing under --dry-run, and reports skipped pages with the reason
  - checked: false
    text: A legacy memory file is deleted only after its page is written and read back
  - checked: false
    text: The wm tools do not repair frontmatter on read
implementation_notes: '2026-11-02 owner decisions: version means the record schema version (no envelope or no schema_version is version 0); migrate is the only migration command and the two old commands become internal steps; migrate writes by default and --dry-run plans; only migrate repairs frontmatter; --dry-run becomes a global flag for all write commands (separate spec and task). Spec approved.'
---

schema_version: 1
state: |-
  wm-cli migrate-memory moves only legacy memory json, and page migrate-records skips 9 pages in eightcap-new-portal because a stub frontmatter block comes first (3 more tasks have criteria only in the stub). Build one ordered migrate command, give every record a version where none means version 0, and stop migrate-memory deleting json it failed to write. Blocked on the owner's answers to the open questions in the spec.
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