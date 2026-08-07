---
title: 'Pattern: Graph Index Staleness Fallback'
type: pattern
id: wiki:patterns:stale-index-disk-fallback
status: reviewed
tags:
- pattern
- tool-reliability
- graph
- mcp
- rust
relates_to:
  - {type: references, target: wiki:tasks:7ce26d}
---

# Pattern: Graph Index Staleness Fallback

> Type: pattern | Tags: [pattern, tool-reliability, graph, mcp, rust]

## Problem

Page-resolution code (update, delete, task handlers) looked up pages **only through the in-memory graph index** and hard-errored with phantom "page not found" when the index was stale — a page on disk but not yet indexed (created externally, or before an index rebuild) became unreachable for writes, while `get` still worked because it had a disk fallback. Symptom: `wm_page.update` / `wm_task.update` return NOT_FOUND for pages that exist and are retrievable via `get`.

## Solution

Resolve page metadata **graph-first, then fall back to disk** via a single shared resolver:

```rust
pub fn resolve_page_meta(engine, id, repo) -> ToolResult<WikiPageMeta> {
    // 1. Fast path: in-memory graph index
    if let Some(node_idx) = engine.graph.load().1.get(id) {
        return Ok(snapshot.0[*node_idx].clone());
    }
    // 2. Stale index: resolve from disk against project_root (mirrors get_page)
    let root = engine.project_root...;
    let file_path = root.join(WM_DIR).join(WIKI_DIR).join(format!("{}.md", path_part));
    let content = repo.read_to_string(&file_path)?;
    Ok(parse_wiki_page(&file_path, &content))
}
```

Key details:
- **Project-root-based path resolution** — the disk fallback must resolve against `engine.project_root`, NOT process CWD (relative `.wm/wiki` paths against CWD silently diverge).
- **One shared resolver** — wire every handler that needs page meta through it (`update_page_with_repo`, task `get`/`update`/`delete`), so graph vs disk behavior never diverges between tools again.
- **ID normalization** — strip `#section` anchors and convert `wiki:type:slug` → `type/slug.md` before joining.

## When to Use

- Any tool/handler that writes (update/delete/link) or reads page data by ID
- Whenever an in-memory index (graph, task store, BM25) may lag the filesystem
- New handlers that must behave identically to `get` on existing pages

## When Not to Use

- Hot paths where the disk read cost matters and the index is guaranteed fresh
- When "page not found" should be a hard error (e.g. creating a brand-new page under an existing ID)

## Related

- @task-7ce26d
- @wiki/concepts/wm-task-store-stale-for-new-pages (symptom + obsolete workaround)
- @wiki/tasks/wmtask-notfound-after-index-rebuild--task-store-loses-tasks-created-via-api