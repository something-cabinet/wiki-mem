---
title: Critical Patterns
type: core
tags: [critical]
status: active
---

---

## 2026-08-10 Frontmatter corruption — never whole-block YAML round-trips

**Category:** pattern / failure
**Source:** @task-wm-task-update-frontmatter-corruption
**Tags:** [yaml, frontmatter, corruption, wiki]

Editing YAML frontmatter by parsing the whole block with serde_yaml and re-serializing corrupts data: unquoted ids like `652e07` become floats (`6520000000.0`), unmodeled fields get dropped, and empty maps emit `{}` blocks. **Always edit frontmatter line-based** (`set_yaml_field`/`remove_yaml_block`/`ac_set_checked` in yaml_helper.rs) and **always double-quote `id`**. Validator rules catch sci-notation ids, duplicate blocks, and id mismatch. A real task file was corrupted this way before the fix; 33 wiki files were repaired.

**Full entry:** @wiki/patterns/line-based-frontmatter-editing

## 2026-08-10 MCP proxy architecture — privileged channel + token split

**Category:** decision
**Source:** @task-22ed6a
**Tags:** [mcp, proxy, architecture, security]

MCP is a thin stdio→HTTP proxy to the wm-server daemon targeting a privileged `POST /api/mcp/tools/{list,call}` channel with a **separate mcp-token** — the web-token surface stays read-only, so a browser-side token leak can never authorize writes. Dynamic `tools/list` from the registry (no STATIC_TOOLS array). Tool errors are HTTP 200 + `{success:false}` mapped to MCP `isError:true`; only auth/transport failures are non-200. ureq in `spawn_blocking`, token re-read + retry-once on 401.

**Full entry:** @wiki/decisions/mcp-proxy-privileged-channel-token-split

## 2026-08-10 Refresh derived state at the write path

**Category:** pattern
**Source:** @task-wm-task-update-frontmatter-corruption
**Tags:** [architecture, graph, cache, stale]

If a write path mutates state that reads consult (graph/index snapshot), refresh the derived snapshot **synchronously in the writer** — never rely on a file watcher to catch up. wm-server runs no watcher, so `wm_task.get` stayed stale until `wm_index_rebuild`; the fix was calling `graph::handle_file_change` after every page write.

**Full entry:** @wiki/patterns/refresh-derived-state-at-write-path

---