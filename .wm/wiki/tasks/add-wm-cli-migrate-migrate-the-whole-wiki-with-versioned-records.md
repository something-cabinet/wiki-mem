---
title: 'Add wm-cli migrate: migrate the whole wiki, with versioned records'
type: task
id: "wiki:tasks:add-wm-cli-migrate-migrate-the-whole-wiki-with-versioned-records"
status: in-progress
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
implementation_plan: |-
  Constraint: wm-core is at 12985 of 13000 lines, so new migration code cannot grow wm-core. Move the fs-only record migration (apps/wm-core/src/page/services/record_migration, 1095 lines) into a new package and keep only engine-bound steps in wm-cli.
  1. Write tests first (TDD): a fixture project with a legacy memory json, a prose page, a stub-block page, a stub-block task with acceptance criteria, a CRLF page, an existing v1 record and a record with no schema_version; assert migrate twice changes nothing the second time.
  2. Add the step framework: page_version() returns 0 when the envelope or schema_version is missing; ordered MigrationStep list moving n to n+1; current version read from wm-engine RECORD_SCHEMA_VERSION.
  3. Step repair-frontmatter: merge a leading stub block into the real block with the line-based helpers (no whole-block YAML round trip), keep acceptance_criteria from the stub, union relates_to, skip with a reason on conflict.
  4. Step import-memory: write the page, read it back, then delete the json; stop ignoring the create_page result; count only verified imports.
  5. Step prose-to-record: reuse the planner and applier; normalize CRLF to LF first; report control-character YAML failures as skips with path and reason.
  6. Step upgrade-version: registry for n to n+1, empty at v1, covered by a test with a fake step.
  7. CLI: Commands::Migrate with local --dry-run replacing MigrateMemory; remove page migrate-records; update record_format_hint.rs and the mcp_test assertion; skipped pages listed, exit non-zero if any; main.rs stays at or under 80 lines, every wm-cli file under 300.
  8. Docs: core/README CLI table, CONVENTIONS, .wm/AGENTS.md, reference/typed-decision-record-schema.
  9. Verify: scripts/check-crate-sizes.sh, check-dependency-layers.sh, check-singleton-helpers.sh, cargo clippy, a dry run on this wiki, then a second run that changes nothing.
time_started: 2026-11-03T09:02:51.405796+00:00
---

schema_version: 1
state: |-
  wm-cli migrate-memory moves only legacy memory json, and page migrate-records skips 9 pages in a real project wiki because a stub frontmatter block comes first (3 more tasks have criteria only in the stub). Build one ordered migrate command, give every record a version where none means version 0, and stop migrate-memory deleting json it failed to write.
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