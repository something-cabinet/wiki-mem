---
id: wiki:specs:strip-all-comments
title: "Strip All Comments from Source Code"
type: spec
status: approved
tags: [spec, refactor, rule-compliance, naming]
acceptance_criteria:
  - "No //, ///, //!, /** */, /* */, or <!-- --> lines remain in apps/* source files"
  - "All TODO comments are replaced by WM tasks"
  - "Build passes (cargo build + ng build)"
  - "Tests pass (cargo test + wdio e2e)"
---
id: wiki:specs:strip-all-comments

## Summary

Remove every comment from all Rust and TypeScript source files. Comments that explain what code does are replaced by extracted named functions or better naming. Section markers and field labels are simply deleted. Doc comments on pub APIs are replaced by self-documenting function names. TODOs are filed as WM tasks.

## Acceptance Criteria

- AC-1: Zero `//` lines in apps/* Rust source
- AC-2: Zero `///` lines in apps/* Rust source
- AC-3: Zero `//!` lines in apps/* Rust source
- AC-4: Zero `//` lines in apps/wm-web/src/ TypeScript source
- AC-5: Zero `/** */` lines in apps/wm-web/src/ TypeScript source
- AC-6: All TODOs replaced by WM tasks
- AC-7: `cargo build -p wm-cli -p wm-core -p wm-server` passes
- AC-8: `cd apps/wm-web && npm run build` passes
- AC-9: `cargo test -p wm-core` passes

## Work Breakdown

### FR-1: wm-cli comments (3 files, ~90 lines)

Files: `main.rs`, `tui.rs`, `mcp_transport.rs`

- Section markers `// ─── ... ───`: delete
- Field label comments: delete
- Explanatory comments: delete or inline into code
- Doc comments `///`: rename functions

### FR-2: wm-server comments (2 files, ~5 lines)

Files: `main.rs`, `routes/graph.rs`

- Simple deletions — these are all executable-step explanations

### FR-3: wm-core/src/mcp/tools/ (20+ files, ~250 lines)

Files: `template/mod.rs`, `task/mod.rs`, `code.rs`, `lsp.rs`, `page/mod.rs`, `memory.rs`, `time.rs`, `doc.rs`, `graph.rs`, `index.rs`, `lint.rs`, `log.rs`, `model.rs`, `project.rs`, `reference.rs`, `search.rs`, `skills.rs`, `source.rs`, `validate.rs`, `version.rs`, `mod.rs`, `page/action.rs`, `page/output.rs`, `task/action.rs`, `task/output.rs`, `decision.rs`

- Section markers `// ─── Action ───`, `// ─── Input types ───`: delete
- Explanatory step comments (`// Skip frontmatter`, `// Apply path filter`): delete or extract
- Doc comments: rename or delete
- Security comments (`// Security: ensure path doesn't escape`): extract into `validate_safe_path()` fn

### FR-4: wm-core/src/engine/ + graph/ + search/ (10+ files, ~120 lines)

Files: `engine/engine_state_mediator.rs`, `engine/main_engine_factory.rs`, `engine/index_scheduler_service.rs`, `engine/write_channel_proxy.rs`, `graph/mod.rs`, `graph/sections.rs`, `search/query.rs`, `search/retrieve.rs`, `search/tests.rs`

- Field labels: delete
- Step comments (`// 1. Get custom types`, `// 2. Full graph rebuild`): extract numbered steps into named helper fns
- Section markers: delete
- Doc comments: rename
- Test comments: delete or convert to assert message strings

### FR-5: wm-core/src/ remaining files (10+ files, ~50 lines)

Files: `error/mod.rs`, `install/mod.rs`, `mcp/prelude.rs`, `mcp/transport.rs`, `page/services/*.rs`, `parser/mod.rs`, `reference_constant.rs`, `reference_service.rs`, `source_service.rs`, `task_board_service.rs`, `util/mod.rs`, `version/version_store_repository.rs`, `config/models/recency_model.rs`

- General deletions
- TODO → task for doc history compaction

### FR-6: TypeScript wm-web/src/libs/graph/ (2 files, ~60 lines)

Files: `canvas-graph.directive.ts`, `graph-color.service.ts`

- Inline what-step comments: extract into named methods (`clearCanvas()`, `drawEdges()`, `drawNodes()`, `drawArrowheads()`, `buildBidirectionalPairs()`, etc.)
- JSDoc: delete or rename function/method
- Section markers: delete

### FR-7: TypeScript wm-web/src/app/ remaining (6 files, ~20 lines)

Files: `views/graph/graph-view.component.ts`, `views/code/code-view.component.ts`, `views/pages/pages-view.component.ts`, `views/search/search-view.component.ts`, `services/api.service.ts`, `libs/ui/utils/src/lib/hlm.ts`, `libs/ui/dialog/src/lib/hlm-dialog.ts`, `libs/ui/sheet/src/index.ts`

- Inline comments: delete or rename
- JSDoc on template-bound methods: keep only the Angular template binding comment as it documents the template contract
