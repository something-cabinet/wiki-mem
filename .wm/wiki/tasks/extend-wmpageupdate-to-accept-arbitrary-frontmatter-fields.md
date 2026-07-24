---
id: wiki:tasks:extend-wmpageupdate-to-accept-arbitrary-frontmatter-fields
title: Extend wm_page.update to accept arbitrary frontmatter fields
type: task
status: todo
priority: medium
tags: [mcp, tool, refactor]
spec: wiki:specs:rename-knownsid-to-id
---
id: wiki:tasks:extend-wmpageupdate-to-accept-arbitrary-frontmatter-fields

`wm_page.update` currently supports only a fixed set of frontmatter fields: title, status, tags, type, relates_to, content, notes. It cannot update arbitrary YAML frontmatter keys, which means:

1. The `knowns_id → id` rename (and any future frontmatter migration) requires bulk file-level sed instead of using the MCP tool
2. No way to programmatically fix frontmatter issues without editing files directly

**Scope:**
- Add an `id` parameter to `wm_page.update` (or a generic `frontmatter: HashMap<String, Value>` field)
- Write the updated frontmatter keys to disk
- Trigger appropriate cache invalidation (stale flag or incremental update)
- Consider exposing a generic `frontmatter` map for future flexibility

Note: This is tool plumbing only — the incremental cache pipeline (graph/BM25/embeddings vs full rebuild) is tracked separately in @wiki/specs/graph-connectivity-fix.