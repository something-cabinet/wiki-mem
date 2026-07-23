---
title: Retire wm_doc — Consolidate onto wm_page
type: spec
status: draft
tags: [spec, refactor, docs, consolidation]
relates_to:
  - {type: references, target: wiki:concepts:edge-types}
---

## Overview

`wm_doc` is fully redundant with `wm_page` — both write to `.wm/wiki/` as `.md` files with YAML frontmatter. `wm_page` supports all 9 page types, typed edges, and richer updates. Remove the `wm-doc` MCP tool, rename `wm-page` to `wm-doc`, and fix the gaps found during review.

## Locked Decisions

- D1: Rename `wm-page` to `wm-doc` (more intuitive name — all existing skills already say `wm_doc.get`)
- D2: Retire the old `wm-doc` MCP tool entirely (no backward compat shim — reindex instead)
- D3: Action-based dispatch stays (`{"action": "create", ...}`) — dot notation in SKILL.md is just user-facing convention
- D4: Tags on create is a bug — must be fixed (currently silently dropped)

## Requirements

### FR-1: Fix `wm_page.create` tags bug
Currently `wm_page.create` accepts `tags` in the JSON schema but discards them (`page.rs` line 162: `tags: _`). Fix: serialize tags into the YAML frontmatter on create, matching what `wm_doc.create` does.

### FR-2: Enrich `wm_page.get` output
`wm_page.get` currently returns only `{id, content, sections}`. Add: `tags`, `type`, `description`, `created_at`, `updated_at` from frontmatter — parity with what `wm_doc.get` returns.

### FR-3: Add graph-index fallback to `wm_page.get`
`wm_doc.create` writes files directly to disk without registering in the graph. `wm_page.get` resolves via graph index and misses these pages. Fix: when graph lookup misses, fall back to filesystem scan, index the page on the fly, then return it.

### FR-4: Remove `wm-doc` MCP tool
Delete or deprecate the `wm-doc` tool registration in `mcp/tools/doc.rs`. All capabilities exist in `wm-page`.

### FR-5: Rename `wm-page` to `wm-doc`
Rename the MCP tool from `wm_page` to `wm_doc`. The `WmPageAction` enum and handler code stay the same — just the registered tool name changes.

### FR-6: Update all 14 skill files
Replace `wm_doc.*` calls with the consolidated `wm_doc.*` (now powered by the old `wm_page`). Update call patterns where arguments differ (`{"action": "create", "type": "..."}` vs `create({"path": "...", ...})`).

### FR-7: Remove redundant learnings from wm-extract
The Consolidation Mode section of wm-extract (C-Step 1 to C-Step 4) scans `learnings/` directory — remove or adapt to use typed pages.

### NFR-1: One-time reindex
Pages created by old `wm_doc` before this change must remain readable. Either the fallback in FR-3 handles this, or run a one-time `wm_index.rebuild` after deploy.

## Acceptance Criteria

- [ ] AC-1: `wm_page.create` writes `tags` into frontmatter (test: create with tags, read back, verify in file)
- [ ] AC-2: `wm_page.get` returns `tags`, `type`, `description`, `created_at`, `updated_at`
- [ ] AC-3: `wm_page.get` finds pages created by old `wm_doc` (fallback works)
- [ ] AC-4: `wm-doc` tool name is removed (or returns deprecation error)
- [ ] AC-5: New `wm_doc` tool name is registered (alias of old `wm_page`)
- [ ] AC-6: All 14 skill files updated — no `wm_doc.create({"path": ...})` calls without `action` dispatch
- [ ] AC-7: `cargo build` passes
- [ ] AC-8: `cargo test` passes with same count
- [ ] AC-9: Existing wiki pages remain readable after change

## Scenarios

### Scenario 1: Create howto page
**Given** a user wants to create a howto page
**When** they call `wm_doc.create({"action": "create", "path": "howto/setup", "type": "howto", "content": "..."})`
**Then** the page is created as a typed howto page with all frontmatter fields, readable via `wm_doc.get`

### Scenario 2: Link extraction to source
**Given** a user just extracted a pattern
**When** they call `wm_doc.link({"id": "wiki:patterns/my-pattern", "target": "wiki:tasks/source-task", "edge_type": "references"})`
**Then** a typed `references` edge is created in the graph, traversable via `wm_graph.neighbors`

### Scenario 3: Read legacy page
**Given** a page was created by the old `wm_doc` (no graph index entry)
**When** `wm_doc.get({"id": "wiki:howto:legacy-page"})` is called
**Then** graph lookup misses, filesystem fallback finds the file, indexes it, and returns content with full metadata

## Technical Notes

### Code changes needed

| File | Change |
|---|---|
| `apps/wm-core/src/mcp/tools/page.rs` | Fix `tags: _` → serialize tags into frontmatter (line ~162). Enrich `Get` output (line ~143). |
| `apps/wm-core/src/mcp/tools/page.rs` | Add filesystem fallback in `Get` handler when graph lookup misses |
| `apps/wm-core/src/mcp/tools/mod.rs` | Remove `doc::register` call. Rename `page::register` to register under `"wm_doc"` name |
| `apps/wm-core/src/mcp/tools/doc.rs` | Delete or reduce to a deprecation shim |
| `apps/wm-core/src/lib.rs` | Remove `pub mod doc` if the file is deleted |
| 14 SKILL.md files | Replace `wm_doc.*` calls with `wm_doc.*` using action dispatch syntax |

### Migration
- One-time `wm_index.rebuild` reindexes all pages into the graph
- OR the filesystem fallback in `wm_doc.get` handles it lazily (preferred — no downtime)

### Edge type reference (consolidated tool)
See @wiki/concepts/edge-types for full reference.
