---
id: wiki:specs:retire-wm-doc
title: Retire wm_doc — Consolidate onto wm_page
type: spec
status: superseded
tags:
- spec
- refactor
- docs
- consolidation
relates_to:
- type: references
  target: wiki:concepts:edge-types
---

---
id: wiki:specs:retire-wm-doc
title: Retire wm_doc — Consolidate onto wm_page
type: spec
status: superseded
tags: [spec, refactor, docs, consolidation]
relates_to:
  - {type: references, target: wiki:concepts:edge-types}
---

# Retire wm_doc — Consolidate onto wm_page

## Status: EXECUTED 2026-08-14 (task wiki:tasks:execute-retire-wm-doc-consolidation)

This spec's locked decisions D1/D2 were **superseded by the executed approach** chosen by the Oracle-reviewed task:

| Spec (draft) | Executed |
|---|---|
| D1: rename `wm-page` → `wm_doc` | **NOT done** — both tool names remain registered (`wm_doc` + `wm_page`) |
| D2: retire old `wm_doc` entirely (no shim) | **NOT done** — `wm_doc` kept as a 152-line alias shim (backward-compatible input schema) |
| FR-1: fix `wm_page.create` tags bug | Already fixed by the issue-126 wave (`c3e465e`) |
| FR-3: graph-index fallback in `get` | Implemented — `doc_get_reads_legacy_file_via_page_path` regression test passes |
| FR-4: remove `wm-doc` tool | **Replaced by** alias-shim consolidation (see @wiki/patterns/alias-shim-consolidation) |

### What was actually done

- `doc.rs` reduced 500 → 152 lines: `WmDocAction` schema unchanged, every action mapped to `WmPageAction` via `to_page_action()`, executed by the extracted shared `page::handle_action()`.
- Deleted: duplicate `parse_frontmatter` (doc.rs:459-486), byte-imitation `build_markdown` (doc.rs:488-499), output structs, `list_docs`, direct `tokio::fs` I/O. Added `confine_doc_path()` preserving historical confinement + audit.
- Parity test `doc_and_page_emit_identical_type_line` (mcp_test.rs:407-429) **deleted** — parity holds by construction (one writer).
- ~361 lines removed net. Verified: mcp_test 53, security_test 18, lib 156, clippy clean.
- One intentional surface change: `wm_doc.get` output now mirrors `wm_page` shapes (`{id, content, sections, tags, type, ...}`) — grep-verified nothing reads the old shapes.

### Remaining from original spec (not done, deferred)

- FR-6 (rename `wm-page` → `wm_doc`) and the 14-skill-file migration: **deferred** — the alias keeps both names working; a full rename is a separate decision if the two-name surface ever becomes a problem.
- FR-7 (wm-extract consolidation mode) — out of scope, unchanged.
- NFR-1 reindex: the filesystem fallback handles legacy pages lazily; no forced rebuild needed.

### Acceptance Criteria (original)

- AC-1 ✅ (tags fixed by #126 wave), AC-2 ✅ (page get enriched — pre-existing), AC-3 ✅ (fallback via alias), AC-4/AC-5 ✅ (alias approach), AC-7/AC-8/AC-9 ✅ (build/test/readable)

## Related

- @wiki/patterns/alias-shim-consolidation — the executed pattern
- @task-execute-retire-wm-doc-consolidation