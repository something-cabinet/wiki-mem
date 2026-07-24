---
id: wiki:specs:wiki-tool-reliability
title: Wiki Tool Reliability — Fix CLI + MCP Bugs
type: spec
---
id: wiki:specs:wiki-tool-reliability

---
id: wiki:specs:wiki-tool-reliability
title: Wiki Tool Reliability — Fix CLI + MCP Bugs
type: spec
tags: [spec, cli, mcp, tooling, bugs]
---
id: wiki:specs:wiki-tool-reliability

## Overview

Multiple bugs in the MCP tool handlers (`wm_page.*`, `wm_index`) and CLI commands (`wm-cli page`) break agent workflows daily, forcing filesystem fallback and violating the MCP-first principle. Root cause in MCP is match-arm patterns that silently discard fields. CLI is missing the `update` command and has path-resolution inconsistencies.

The project follows an **AI-agent-first** model — CLI page commands exist for MCP tool handlers, not interactive human use. All content I/O is via stdin.

## Locked Decisions

- D1: Scope covers ALL wiki tool reliability bugs — MCP (match arms, id confusion, undocumented params) + CLI (no update, multiline content, meta.path relative) + wm_index
- D2: Fix + refactor to typed handler pattern that prevents field discarding + regression tests per bug
- D3: Remove `--content` flag, always use stdin for page content
- D4: CLI must be invoked from wiki root. `meta.path` is always resolved relative to CWD (which IS the wiki root)
- D5: Add `wm-cli page update` subcommand
- D6: `page create` and `page update` accept content via stdin only (no `--content` flag)
- D7: Include wm_index reliability bugs in scope
- D8: Each bug fix must include integration + regression tests

## Requirements

### Functional Requirements

#### FR-1: Fix MCP match-arm discarding
In `mcp/tools/page.rs`, fix pattern matches that use `_` to discard fields like `tags`. Each field must be explicitly bound or processed. Apply across all page tool action handlers.

#### FR-2: Fix `page_id` / `id` parameter confusion
Unify parameter naming across all MCP page tools. Use `id` consistently. Ensure JSON schemas reflect the actual required parameters.

#### FR-3: Add missing input JSON schemas
All MCP page tool parameters must have complete JSON schemas. Undocumented required params cause agents to guess and fail.

#### FR-4: Fix wm_index reliability bugs
Audit and fix bugs in the `wm_index` tools (rebuild, status, etc.) that cause phantom "page not found" errors and other reliability failures.

#### FR-5: Add `wm-cli page update` subcommand
New CLI command mirroring MCP `wm_page.update`:

```bash
wm-cli page update <id>
  # accepts JSON via stdin with optional fields:
  # { "title": "...", "content": "...", "status": "...", "tags": [...], "type": "..." }
```

#### FR-6: Stdin-only content input
Remove `--content` flag from `page create`. Both `page create` and `page update` accept content exclusively via stdin. This fixes multiline breakage naturally.

#### FR-7: Path resolution consistency
All CLI page commands (get, create, update, delete, link, unlink) must resolve `meta.path` consistently relative to CWD (which equals wiki root per D4). Fix the `link`/`update`/`delete` commands that resolve against the wrong base.

#### FR-8: Typed handler pattern (D2)
Refactor MCP page tool handlers to a typed dispatch pattern that structurally prevents field discarding. Each action variant must explicitly declare which fields it processes.

### Non-Functional Requirements
- NFR-1: All existing integration tests must pass
- NFR-2: Each bug fix includes a regression test that fails before the fix and passes after
- NFR-3: New CLI `page update` must have E2E integration test
- NFR-4: All MCP tool JSON schemas must be complete and accurate

## Acceptance Criteria

- [ ] AC-1: `wm_page.get` with valid `id` returns complete page data (tags, title, content, status, type)
- [ ] AC-2: `wm_page.update` with `tags` field correctly sets page tags (previously discarded)
- [ ] AC-3: `wm_page` tools accept `id` parameter (not `page_id`) — both forms work, `id` canonical
- [ ] AC-4: All `wm_page.*` tool JSON schemas show required parameters correctly
- [ ] AC-5: `wm_index rebuild` completes without phantom "not found" errors
- [ ] AC-6: `wm-cli page update <id>` reads JSON from stdin and updates page correctly
- [ ] AC-7: `wm-cli page create` with stdin content creates page with correct multiline body
- [ ] AC-8: `wm-cli page link/update/delete` resolve paths correctly from wiki root CWD
- [ ] AC-9: No match-arm uses `_` to discard action fields in page tool handlers
- [ ] AC-10: Each bug in the bug list has a regression test
- [ ] AC-11: All existing integration tests pass

## Bug List

| # | Tool | Symptom | Root Cause |
|---|------|---------|------------|
| B1 | `wm_page.update` | `tags` parameter silently ignored | Match arm: `tags: _` instead of `tags` |
| B2 | `wm_page.*` | Phantom "page not found" on update | `tags: _` discard causes stale state |
| B3 | `wm_page.*` | `page_id` vs `id` parameter confusion | Ambiguous naming, 2 params for same thing |
| B4 | `wm_page.*` | Missing required parameter schemas | Undocumented in JSON schema |
| B5 | `wm-cli page` | No `update` command | Never implemented |
| B6 | `wm-cli page` | `--content` breaks on multiline | Flag not suitable for multiline text |
| B7 | `wm-cli page link/update/delete` | `NOT_FOUND` — path resolved wrong | `meta.path` relative to wrong base |
| B8 | `wm_index` | Phantom "page not found" on index rebuild | Underlying page lookup issue |

## Scenarios

### Scenario 1: Agent updates page tags
**Given** an agent calls `wm_page.update` with `{ "id": "my-page", "tags": ["rust", "async"] }`
**Then** the page's tags are set to `["rust", "async"]` (not silently discarded)

### Scenario 2: Agent creates page with multiline content
**Given** an agent pipes multiline content via stdin to `wm-cli page create`
**Then** the page is created with the complete multiline body (not truncated at first newline)

### Scenario 3: CLI page update from wiki root
**Given** the CWD is the wiki project root
**When** `wm-cli page update my-page` is invoked with stdin JSON
**Then** the page is found and updated correctly (no `NOT_FOUND` error)

## References
- `apps/wm-core/src/mcp/tools/page/` — MCP page tool handlers
- `apps/wm-cli/src/main.rs` — CLI page commands
- `apps/wm-core/src/page/services/page_update_builder_service.rs` — Update logic
- `apps/wm-core/src/page/helpers/page_path_helper.rs` — Path resolution
- `apps/wm-cli/tests/` — CLI integration tests
