---
title: 'Wiki Tool Reliability: wm_doc.get core-doc bootstrap guidance — id vs path, plus core/* session loading'
type: task
id: "wiki:tasks:wiki-tool-reliability-wmdocget-core-doc-bootstrap-guidance--id-vs-path-plus-core-session-loading"
status: in-review
priority: high
tags: [tool-reliability, docs, guidance, session-context]
acceptance_criteria:
  - text: "WIKI-MEM.md (canonical + embedded shim) instructs loading core docs from core/* (core/README, core/ARCHITECTURE, core/CONVENTIONS, core/critical-patterns) with correct tool params"
  - text: "wm-init skill Step 3 reads core/* instead of wm_doc.get({id:\"README\"}); Step 5 uses path + type:\"page\""
  - text: "No remaining `wm_doc.* id` or `type:\"doc\"` occurrences in any embedded skill or .opencode skill copy"
  - text: ".wm/AGENTS.md and the main.rs generator document the record body envelope (schema_version/state/questions/answers)"
  - text: "cargo check/test for touched crates passes; grep guard returns zero broken patterns"
---

**Bug description:** Every agent-facing instruction bootstraps by reading core docs with `wm_doc.get({"action":"get","id":"README"})` (plus ARCHITECTURE/CONVENTIONS and the learning path). The `wm_doc` adapter in `apps/wm-core/src/mcp/tools/doc.rs` (WmDocAction::Get { path }) requires `path` and derives the id via `path_to_id()`; it never accepts `id`. Every one of these calls fails at deserialization: SERDE_ERROR `missing field \`path\``.

**Tool name:** `wm_doc` (get/update/delete variants)

**Full input parameters:** `{"action":"get","id":"README"}`

**Full error output:** `Error: {"code":"SERDE_ERROR","error":"deserialize tool input failed: missing field `path`"}`

**Counter-evidence:** `wm_doc.get({"action":"get","path":"core/README"})` returns content; `wm_page.get({"action":"get","id":"wiki:core:README"})` returns content.

**Workaround:** Use `wm_page.get` with `id`, or `wm_doc.get` with wiki-relative `path` (e.g. `core/README`), or read `.wm/wiki/<path>.md` directly.

**Reproduction steps:** 1) Call `wm_doc.get({"action":"get","id":"README"})` -> SERDE_ERROR. 2) Call `wm_doc.get({"action":"get","path":"core/README"})` -> success.

**Same defect class (all fixed in this pass):**
- `wm_doc.get/update/delete` with `id` in `wm-init`, `wm-doc`, `wm-debug`, `wm-plan`, `wm-flow`, `wm-review`, `wm-template`, `wm-validate`, `wm-extract`, `wm-go` skills (embedded + `.opencode` copies).
- `wm_search.query({"type":"doc"})` — invalid SearchType (valid: all|page|task|memory); should be `page`.
- `WIKI-MEM.md` carried no core-doc session-loading instruction at all, and `.wm/AGENTS.md` (generated at `wm-cli/src/main.rs`) still documents the pre-record frontmatter schema. Residual finding: `wm_search.query({"type":"doc"})` uses dotted pseudo-syntax (`wm_search.query(...)`) that does not match the actual `wm_search_query` / `wm_search` tool + `action` shape.