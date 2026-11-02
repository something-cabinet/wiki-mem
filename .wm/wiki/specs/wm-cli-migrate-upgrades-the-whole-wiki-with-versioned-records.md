---
title: wm-cli migrate upgrades the whole wiki, and records carry a version
type: spec
id: "wiki:specs:wm-cli-migrate-upgrades-the-whole-wiki-with-versioned-records"
status: draft
tags: [cli, migration, records, versioning, spec]
acceptance_criteria:
  - text: wm-cli migrate runs every wiki migration in order and is idempotent
  - text: It repairs pages whose first frontmatter block is a stub, merging the blocks so no field is lost
  - text: It converts legacy memory json and prose bodies to records
  - text: Every record carries a version; a record or page with none is version 0 and is upgraded step by step to the current version
  - text: It plans by default and writes only with an explicit apply flag, and reports skipped pages with the reason
  - text: A legacy memory file is deleted only after its page is written and read back
---

schema_version: 1
state: |-
  ## Goal

  One command, wm-cli migrate, brings a project wiki to the current format. Today the work is split across commands that each cover a slice and leave the rest broken.

  ## Verified current state

  - migrate-memory moves .wm/memory/*.json into memory pages and nothing else. It ignores the result of create_page, deletes the json anyway and counts the entry as migrated (apps/wm-core/src/page/services/migration_service.rs).
  - page migrate-records converts prose bodies to the record envelope. In eightcap-new-portal its dry run skipped 9 pages (concept 1, decision 1, pattern 6, spec 1) with: frontmatter block 0 is not a mapping.
  - Those pages, and 3 task pages the same run does not list, begin with a stub block (a line with {} plus tags, or an empty block) followed by the real frontmatter. They come from the 2026-07-30 Knowns import. The parser reads the first block, so title, type and id in the second block are ignored.
  - For the 9 non-task pages the stub holds nothing the real block lacks. For the 3 tasks the acceptance_criteria exist only in the stub, so the blocks must be merged, not the stub dropped.
  - Records already carry schema_version: 1 in the envelope. A page with no envelope or no schema_version has no version at all.

  ## Change

  - Add wm-cli migrate with ordered steps: repair frontmatter, import legacy memory, convert prose to records, upgrade record versions.
  - A page or record without a version is version 0. Each step moves a page from version n to n+1, so a later format change adds one step and old projects upgrade in order.
  - Plan first, write only with an apply flag, as page migrate-records does today. Report every skipped page and why.
  - Memory import writes the page, reads it back, and only then deletes the json.

  ## Open questions for the owner

  - Version meaning: the schema version of the record envelope (the reading used above, where version 0 means no envelope), or a revision counter of the page content that rises on every edit. They are different features.
  - Old commands: keep migrate-memory and page migrate-records as aliases of the matching steps, or remove them.
  - Default: plan-only unless apply is given (recommended, matches migrate-records), or apply by default.
  - Should the wm tools also repair a stub block on read, or only this command.

  ## Out of scope

  Changing what the engine does at startup, and the root cause that wrote the stub blocks (issue 139).

  ## Verification

  - Tests, written first, on a project containing a legacy memory json, a prose page, a stub-block page and a stub-block task with criteria.
  - Running migrate twice changes nothing the second time.
  - Running it on eightcap-new-portal leaves 0 skipped pages.
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
