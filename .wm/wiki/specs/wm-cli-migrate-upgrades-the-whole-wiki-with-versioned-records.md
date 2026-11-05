---
title: wm-cli migrate upgrades the whole wiki, and records carry a version
type: spec
id: "wiki:specs:wm-cli-migrate-upgrades-the-whole-wiki-with-versioned-records"
status: approved
tags: [cli, migration, records, versioning, spec]
acceptance_criteria:
  - text: wm-cli migrate migrates the entire wiki in order and is idempotent
  - text: migrate-memory and page migrate-records are gone from the CLI and live on as internal steps of migrate
  - text: It repairs pages whose first frontmatter block is a stub, merging the blocks so no field is lost
  - text: It converts legacy memory json and prose bodies to records
  - text: Every record carries a version; a record or page with none is version 0 and is upgraded step by step to the current version
  - text: It writes by default and writes nothing under --dry-run, and reports skipped pages with the reason
  - text: A legacy memory file is deleted only after its page is written and read back
  - text: The wm tools do not repair frontmatter on read
---

schema_version: 1
state: |-
  ## Goal

  One command, wm-cli migrate, migrates the entire wiki to the current format. Today the work is split across commands that each cover a slice and leave the rest broken.

  ## Decisions (owner)

  - Version: the schema version of the record envelope. A page with no envelope, or an envelope without schema_version, is version 0.
  - Commands: wm-cli migrate is the only migration command. migrate-memory and page migrate-records are removed from the CLI and stay as internal steps of migrate.
  - Default: migrate writes. It plans without writing only when --dry-run is given.
  - Scope: only migrate repairs a stub frontmatter block. The wm tools do not repair on read.
  - Dry run: --dry-run becomes a global flag for every write command. That is a separate spec (wm-cli global dry-run flag).
  - Code home (2026-11-03): the file-system-only migration steps live in a new package, because wm-core is 15 lines under its 13000-line ceiling. Steps that need the engine, such as the memory import, stay in wm-cli.

  ## Verified current state

  - migrate-memory moves .wm/memory/*.json into memory pages and nothing else. It ignores the result of create_page, deletes the json anyway and counts the entry as migrated (apps/wm-core/src/page/services/migration_service.rs).
  - page migrate-records converts prose bodies to the record envelope and plans unless --apply is given. In a real project wiki its run skipped 9 pages (concept 1, decision 1, pattern 6, spec 1) with: frontmatter block 0 is not a mapping.
  - Those pages, and 3 task pages the same run does not list, begin with a stub block (a line with {} plus tags, or an empty block) followed by the real frontmatter. They come from the 2026-07-30 Knowns import. The parser reads the first block, so title, type and id in the second block are ignored.
  - For the 9 non-task pages the stub holds nothing the real block lacks. For the 3 tasks the acceptance_criteria exist only in the stub, so the blocks must be merged, not the stub dropped.
  - Records already carry schema_version: 1 in the envelope. A page with no envelope or no schema_version has no version at all.
  - Two pages in this wiki fail to parse: one has a carriage return in its state, one has a control character in its record YAML.

  ## Change

  - Add wm-cli migrate with ordered steps: repair frontmatter, import legacy memory, convert prose to records, upgrade record versions.
  - Each step moves a page from version n to n+1, so a later format change adds one step and old projects upgrade in order.
  - Remove the migrate-memory command and the page migrate-records subcommand from the CLI. Their code becomes steps of migrate.
  - Memory import writes the page, reads it back, and only then deletes the json.
  - Normalize CRLF to LF before converting a body. A control character in YAML is not repaired; the page is reported as skipped.
  - Report every skipped page and why. Exit non-zero when any page could not be migrated.

  ## Out of scope

  The global --dry-run flag, changing what the engine does at startup, and the root cause that wrote the stub blocks (issue 139).

  ## Verification

  - Tests, written first, on a project containing a legacy memory json, a prose page, a stub-block page and a stub-block task with criteria.
  - Running migrate twice changes nothing the second time.
  - Running it on a real project wiki that held stub-block pages leaves 0 skipped pages.
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
  kind: migration
  scope: project-wide
  needs_tasks: true