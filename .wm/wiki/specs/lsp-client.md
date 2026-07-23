---
title: LSP Client — Code Intelligence via Language Server Protocol
type: spec
tags: [spec, lsp, code-intel, mcp]
---

## Overview

Build a full LSP client in Rust (`packages/wm-lsp`) that discovers, spawns, and manages language servers. Exposed via MCP tools (`wm_lsp.*`) through the wm-server singleton daemon. Replaces the current tree-sitter-only code intelligence with real cross-file references, definitions, diagnostics, and rename.

## Locked Decisions

- D1: 4 languages first — Rust (rust-analyzer), Go (gopls), TypeScript (typescript-language-server), Python (pyright/pylsp)
- D2: Hybrid LS discovery — check PATH, then known install locations (rustup, npm global), report install hints when not found
- D3: Write-through proxy — central notification point for all file mutations, keeping LSP in sync with external edits
- D4: Async dispatch — add `register_typed_async` + `dispatch_async` to ToolRegistry; existing sync handlers untouched
- D5: Phase 1 scope — definition, references, hover, workspace_symbols, implementations, diagnostics (diagnostics cached)
- D6: Tree-sitter fallback for workspace_symbols when LS unavailable; other tools return error with install hint
- D7: Idle TTL — shut down LS after 30 minutes without queries (configurable)
- D8: Rename dry-run by default; `apply: true` required to execute

## Requirements

### Functional Requirements

**FR-1: LSP Transport**
Implement Content-Length framed JSON-RPC transport over child process stdio. Reader task demuxes responses by request ID using `DashMap<RequestId, oneshot::Sender>`. Writer via `mpsc` channel. Supports concurrent in-flight requests per server.

**FR-2: LS Lifecycle**
Initialize → Initialized → (queries) → Shutdown → Exit. Handle server-initiated requests with appropriate responses.

**FR-3: Language Server Discovery**
For each of the 4 target languages: check PATH, then known install locations (rustup for rust-analyzer, npm global for typescript-language-server, etc.), then return structured error with install hint.

**FR-4: File Synchronization**
Write-through proxy: all file mutations route through a central notification point that sends `didChange` to relevant LSP sessions.

**FR-5: Definition**
`wm_lsp.definition` — return target file, line, column, and snippet.

**FR-6: References**
`wm_lsp.references` — all locations referencing the symbol.

**FR-7: Hover**
`wm_lsp.hover` — markdown contents with optional range.

**FR-8: Workspace Symbols**
`wm_lsp.workspace_symbols` — matching symbols across workspace. Falls back to tree-sitter when LS unavailable.

**FR-9: Implementations**
`wm_lsp.implementations` — all implementations of the interface/trait.

**FR-10: Diagnostics**
`wm_lsp.diagnostics` — cached diagnostics, filterable by file and severity.

**FR-11: Rename**
Dry-run by default (returns edit plan). `apply: true` executes.

**FR-12: Status**
`wm_lsp.status` — per-language server status.

**FR-13: Tree-sitter Fallback**
workspace_symbols falls back to tree-sitter. Other tools error with install hint.

**FR-14: Readiness Gating**
Return structured `starting` status when LS is indexing.

**FR-15: Lazy Start**
Language servers start on first query to that language.

**FR-16: Idle Shutdown**
Shut down after 30 minutes without queries (configurable).

### Non-Functional Requirements
- NFR-1: Warm definition <2s. Cold start signals `starting` within 200ms, result within 30s.
- NFR-2: Concurrent queries to different language servers do not block each other.
- NFR-3: UTF-16/UTF-8 position conversion correct for CJK and emoji.
- NFR-4: Rename dry-run never writes to disk.
- NFR-5: LSP feature gated behind `lsp` feature flag.
- NFR-6: Windows support: URI drive-letter encoding, `.exe` resolution.

## Acceptance Criteria

- [ ] AC-1: `wm_lsp.definition` on a Rust symbol returns correct file/line/col/snippet
- [ ] AC-2: `wm_lsp.references` on a function returns all call sites
- [ ] AC-3: `wm_lsp.hover` on a symbol returns meaningful markdown
- [ ] AC-4: `wm_lsp.workspace_symbols` returns matches from all 4 languages
- [ ] AC-5: workspace_symbols falls back to tree-sitter when LS unavailable
- [ ] AC-6: `wm_lsp.diagnostics` returns cached diagnostics, filterable by severity
- [ ] AC-7: `wm_lsp.rename` dry-run returns edit plan without writing files
- [ ] AC-8: `wm_lsp.rename` with `apply: true` writes changes to disk
- [ ] AC-9: `wm_lsp.status` shows enabled/disabled, binary found, running/stopped, readiness
- [ ] AC-10: LS not found returns structured error with install hint
- [ ] AC-11: Readiness timeout returns `starting` status without blocking
- [ ] AC-12: Idle LS shuts down after TTL and restarts on next query
- [ ] AC-13: File edits via MCP tools refresh LSP buffers automatically
- [ ] AC-14: All 4 language servers can start and serve queries
- [ ] AC-15: 10 concurrent definitions to the same LS produce correct results
- [ ] AC-16: UTF-16/UTF-8 conversion handles CJK and emoji correctly

## Scenarios

### Scenario 1: Symbol definition
**Given** a rust-analyzer session is running
**When** an agent calls `wm_lsp.definition` with file path and symbol query
**Then** it returns the file, line, column, and code snippet of the definition

### Scenario 2: Rename with dry-run
**Given** an LSP session on a TypeScript project
**When** an agent calls `wm_lsp.rename` with position and new_name
**Then** dry-run returns all files and ranges that would change
**And** `apply: true` executes the rename

### Scenario 3: LS not installed
**Given** gopls is not on PATH
**When** an agent calls `wm_lsp.definition` on a Go file
**Then** it returns `{code: "unavailable", language: "go", install_hint: "..."}`

### Scenario 4: External file edit
**Given** an LSP session is active
**When** an agent edits a file via `wm_page.update`
**Then** the write-through proxy sends `didChange` to LSP
**And** next `wm_lsp.diagnostics` returns up-to-date results

## Technical Notes

### Crate structure
```
packages/wm-lsp/
├── Cargo.toml
└── src/
    ├── lib.rs         # LspManager, pub exports
    ├── error.rs       # LspError
    ├── transport.rs   # Content-Length JSON-RPC, reader demux, writer channel
    ├── client.rs      # typed request API
    ├── server.rs      # LspServer: process + client + readiness
    ├── process.rs     # spawn/monitor LS child process
    ├── filesync.rs    # didOpen/didClose with ref-counting
    ├── manager.rs     # DashMap<LangId, Arc<LspServer>>
    ├── detect.rs      # LS binary discovery
    ├── adapters.rs    # per-language configs
    ├── position.rs    # UTF-8 ↔ UTF-16 conversion
    └── uri.rs         # path ↔ file:// URI
```

### Integration points
- `LspManager` added to `EngineState` as `pub lsp: Arc<LspManager>`
- Feature-gated: `lsp = ["dep:wm-lsp"]` in wm-core
- Enabled by default in wm-server, disabled in wm-cli direct mode
- MCP tools via `register_typed_async` in ToolRegistry
- Proxy tools: `wm_lsp.*` added to `STATIC_TOOLS`

## References
- `apps/wm-core/src/mcp/transport.rs` — ToolRegistry (add async dispatch)
- `apps/wm-core/src/engine/engine_state_mediator.rs` — EngineState (add LspManager)
- `apps/wm-core/src/config/models/lsp_settings_model.rs` — LspLanguageSettings
- `packages/wm-code-intel/src/lib.rs` — tree-sitter fallback
- `apps/wm-cli/src/mcp_proxy.rs` — STATIC_TOOLS
- `apps/wm-server/src/routes/tools.rs` — generic tool dispatch

## Open Questions

- [ ] Should `wm_lsp.rename` support WorkspaceEdit.documentChanges in addition to changes?
- [ ] Mtime-check aggressiveness: on every query, or only if > N seconds since last check?
