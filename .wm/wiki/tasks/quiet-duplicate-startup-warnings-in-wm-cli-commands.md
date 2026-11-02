---
title: Quiet duplicate startup warnings in wm-cli commands
type: task
id: "wiki:tasks:quiet-duplicate-startup-warnings-in-wm-cli-commands"
status: done
priority: medium
tags: [cli, logging, parser]
spec: wiki:specs:quiet-wm-cli-startup-warnings
acceptance_criteria:
  - text: "Each distinct broken page produces at most one parser warning that names its file"
    checked: true
  - text: "migrate-memory in a project with no .wm/memory builds no engine and prints no engine warnings"
    checked: true
  - text: "index rebuild builds the graph once per run"
    checked: true
  - text: "Tests cover the warning dedupe and the migrate-memory no-engine path"
    checked: true
implementation_notes: |-
  2026-11-02: owner chose options A+B. A: packages/wm-engine/src/parser/mod.rs now reports one warning per source and reason per process (warn_frontmatter_once), the field and YAML errors travel as the reason instead of being logged separately, and extract_frontmatter (no source) is silent. B: migrate-memory only builds an engine when .wm/memory holds a json entry (has_legacy_memory_entries). Found and fixed on the way: migrate-memory reported 0 when build_engine had already auto-migrated the entries; it now migrates once and reports the real count. Tests: packages/wm-engine/tests/frontmatter_warnings.rs (6), apps/wm-core/tests/migrate_memory_test.rs (2), written red first. Measured in eightcap-new-portal: migrate-memory 151 lines and 78 WARN -> 1 line and 0 WARN; index rebuild --skip-embed 301 lines and 155 WARN -> 32 lines and 23 WARN (12 parser, one per broken page, 10 graph lines because the graph is built twice, 1 vectors.db lock). Open: acceptance criterion 3 (index rebuild builds the graph once) is not done; call_tool builds and refreshes the engine before wm_index_rebuild rebuilds it again. check_ac through the 0.5.2 MCP server returned success without persisting (issue 140); the three checks were applied through the 0.6.1 server and read back.
  Criterion 3 done: index rebuild now dispatches through call_rebuilding_tool, which skips the startup index refresh (StartupRefresh::Skip), so the graph is built once. Test: apps/wm-core/tests/index_rebuild_test.rs index_rebuild_builds_the_graph_once, written red first (2 occurrences, then 1).
---

Measured with wm-cli 0.6.1 in ~/projects/eightcap-new-portal (225 pages, 12 with unparseable frontmatter). migrate-memory prints 78 WARN lines and migrates 0 entries. index rebuild --skip-embed prints 155 WARN lines (144 parser, 5 unresolved relates_to, 1 vectors.db lock) and logs Graph rebuilt twice. Cause 1: the parser logs two warnings per failing parse (one without the file name) and every pass re-parses. Cause 2: migrate-memory builds the full engine (ONNX, vectors.db, graph, BM25) to migrate nothing. See the spec for options A, B, C.