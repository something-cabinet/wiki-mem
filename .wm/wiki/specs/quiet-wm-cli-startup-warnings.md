---
title: Quiet duplicate startup warnings in wm-cli commands
type: spec
id: "wiki:specs:quiet-wm-cli-startup-warnings"
status: approved
tags: [cli, logging, parser, spec]
acceptance_criteria:
  - text: A broken page produces at most one parser warning per process, naming its file and reason
  - text: extract_frontmatter without a source never warns
  - text: migrate-memory builds no engine when .wm/memory holds no json entry
  - text: migrate-memory reports the real count when entries exist
---

schema_version: 1
state: |-
  ## Goal

  A command that has nothing to do must not print dozens of warnings, and a single broken page must produce one warning, not six.

  ## Verified current state

  Measured with wm-cli 0.6.1, `wm-cli migrate-memory` in ~/projects/eightcap-new-portal (225 pages, no .wm/memory directory):

  - 151 output lines, 78 of them WARN, and the result is "Migrated 0 memory entries".
  - 12 distinct pages fail to parse, yet the parser emits 72 warnings: 33 "Frontmatter YAML parse failed", 24 "Frontmatter parse error in <unknown source>", 12 "Frontmatter parse error in <named file>", and 3 "Frontmatter is not a mapping: Null".
  - 5 "unresolved relates_to" warnings from the graph and 1 vectors.db lock warning.

  ## Causes

  1. Duplication. packages/wm-engine/src/parser/mod.rs logs twice per failing parse: once in build_page_meta (YAML parse failed or not a mapping, no file name) and once in extract_frontmatter_from (parse error in <source>). Callers that use extract_frontmatter pass "<unknown source>", so the second line names no file. Each page is also parsed by several passes (graph rebuild, section build, index), so the pair repeats.
  2. Unneeded work. apps/wm-cli/src/migrate_memory/migrate_memory_service.rs calls create_engine, which loads the ONNX embedder, opens vectors.db, rebuilds the graph and the BM25 index, and then migrates. build_engine already auto-migrates when .wm/memory exists, so the command repeats work every other command does.

  ## Options

  - A. Log one warning per file per process, with the file name, and fold the two parser messages into one. Recommended: fixes the duplication for every command.
  - B. Make migrate-memory migrate directly without building the engine. Recommended: removes the 78 warnings for this command and the ONNX load.
  - C. Print a one-line summary ("12 pages skipped, run wm-cli lint check") at default level and move per-file detail to debug. Alternative to A; hides detail by default.

  ## Out of scope

  Repairing the broken pages in eightcap-new-portal, and the root cause of the leading {} frontmatter block (issue #139).

  ## Verification

  - Same command in the same project prints at most one warning per distinct broken page, each naming its file.
  - migrate-memory in a project with no .wm/memory prints no engine warnings.
  - Tests cover the dedupe and the no-engine path.
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
  kind: system
  scope: component
  needs_tasks: true