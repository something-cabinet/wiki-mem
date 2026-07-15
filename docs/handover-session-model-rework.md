# Session Handover — WM Model Rework Complete

## Session Summary

Massive session covering the full WM model rework, Knowns parity filling, CDD fixes, E2E tests, and documentation.

## Accomplishments

### Model Layer
- `enum Page` with typed variants (Task, Spec, Decision, Pattern, Memory, Concept, etc.)
- `TaskData`, `SpecData`, `DecisionData`, `PatternData`, `MemoryData` per-type structs
- `MemoryStatus` (Active, Stale, Archived) + `MemoryLayer` (Project, Global, Session) enums
- `PageType::as_str()` — removed 9 Debug-format call sites
- `PageType::allowed_statuses()` — per-type validation at tool boundary
- `published: bool`, `order: Option<i32>` on WikiPageMeta
- `Memory` page variant — memory entries are now wiki pages in `.wm/wiki/memory/`

### MCP Tool Surface
- 78→46 tools merged (typed.rs deleted, 225 LOC removed)
- Action enum dispatch for 10 merged domains
- `register_typed()` replaces `register_read/write/admin`
- `ToolError::invalid_params()` added

### Config
- `PermissionPreset` (ReadWrite, ReadOnly), `RecencyModel` (Fsrs, Linear, Exponential, None)
- `SearchConfig.default_mode` uses `SearchMode` enum
- `StatusColors`, `visible_columns`, `LspLanguageSettings`, `GitTracking`, `runtime_memory_*`
- Config consumers wired: statusColors + visibleColumns filter task board

### CDD Fixes
- `relates_to: Vec<String>` → `Vec<(EdgeType, String)>` with custom YAML deser
- `PageUpdateParams` — removed `serde_json::Value` round-trip
- `impl Display` for PageStatus, Priority
- Tool input structs use typed enums over `Option<String>`

### Template System
- `_template.yaml` directory format with prompts + actions
- Actions implemented: add, addMany, modify, append
- `when` condition evaluation, `skip_if_exists` support
- Backward compat with old `.json` templates

### Version History
- `VersionStore` with task/doc version save/get/rollback
- FSRS-driven compaction
- `wm_version.list`, `wm_version.get`, `wm_version.rollback` MCP tools
- Automatic version capture on page/task update

### Memory→Page Migration
- Memory entries now wiki pages in `.wm/wiki/memory/`
- `search/memory.rs` gutted (memory indexed via main pipeline)
- `migrate_old_memory_json()` — one-time migration from old `.json` format
- Session memory stays in-memory (DashMap), config-driven capacity + FSRS eviction

### Vector Storage (Turso)
- `turso` crate replaces `wm-vectors-bin` (pure Rust, no C compiler)
- `VectorDb` module with background thread for safe tokio interop
- `run_async()` bridge using `block_in_place` + `Handle::current().block_on()`
- Chunks + content_hashes tables, incremental rebuild

### References
- Format changed from `@doc/`, `@task-`, `@memory/`, `@decision/`, `@template/` to `@wiki/{type}/{name}`
- 27 wiki files migrated (118+ reference replacements)
- `resolve_reference()` simplified to unified wiki page lookup

### Tests
- **207 tests total** — 112 unit + 31 CLI + 3 E2E + 10 E2Ev2 + 51 MCP
- All pass, zero failures
- 23 new tests added (14 unit + 9 MCP)
- 10 new E2E tests covering memory, vectors, versions, templates, config, refs, status validation

### Documentation
- `WIKI-MEM.md` aligned with KNOWNS.md structure (cleaner TL;DR, References, File Roles, CLI pitfalls)
- Wiki conventions/workflows stripped from WIKI-MEM.md (moved to skills)
- `search-scoring-formula.md` — 10 walkthroughs with concrete tables
- `scoring-config.md` — 5 updated descriptions with examples
- `README.md` — BM25/semantic/RRF definitions
- `@doc/specs/` — 8 spec documents created

### Cleanup
- `.knowns/` directory deleted (legacy Knowns data)
- `wm-ui/` dead SvelteKit project deleted (Angular `apps/wm-web/` is the web UI)
- `.knowns` removed from code.rs exclusion list
- `spec.md` wiki entry created at `.wm/wiki/specs/local-knowledge-engine.md`

## Remaining Tasks

### High Priority
1. **Web UI polish** (`task-web-ui-polish-production-readiness`) — Memory create stub, status filter, error handling in Graph/Tasks/Memory/Settings, responsive sidebar
2. **LSP/git tracking config** (`task-lsp-git-config-consumers`) — Verify wiring is complete

### Low Priority
3. **Stress tests** (`task-stress-scale-tests`) — 1000 page graph, 10K doc search, concurrent MCP

### Deferred (from earlier decisions)
- Workspace (agent execution) — deferred D9
- Chat (conversation persistence) — deferred D10
- Turso cloud sync — not in scope (local-only)
