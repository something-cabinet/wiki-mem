---
implementation_notes: 'Additional evidence 2026-07-31: task wiki:tasks:wm-index-code-output-misleading--report-totals-make---skip-hash-check-force-re-parse went through wm_task.update(status, implementation_plan, append_notes) x3 during its lifecycle. Final file frontmatter contained ONLY status: done + implementation_notes — id:/title:/type: were stripped by the update path (same root cause as this issue). Verify wm_task.update preserves all frontmatter fields.'
---

## Bug Description

`wm_task.update` behaves unreliably on task status transitions AND corrupts task frontmatter:

1. **Invalid transition errors**: `wm_task.update({action:"update", id, status:"done"})` on a `todo` task returns `INTERNAL_ERROR: Invalid transition: todo → done. Allowed: in-progress, cancelled`. Updating to `in-progress` first reports success (`{"id":...,"status":"updated"}`) but a subsequent `get` still shows `todo` — the transition validation reads stale state, and even after a successful in-progress update the file is left inconsistent.
2. **Frontmatter corruption**: after `wm_page.link` (adding `relates_to` edges) plus `wm_task.update` calls, the 8 affected task files had their frontmatter mangled — `id`, `title`, `type`, `tags`, `priority` dropped and replaced with a stray `{}` block:
   ```markdown
   ---
   {}
   relates_to:
     - {type: implements, target: wiki:specs:wiki-tool-reliability}
   ---
   ```
   This made `wm_task.get` return `NOT_FOUND` for pages that exist on disk and `wm_task.list` (by label) return empty results.

## Tool Name

`wm_task.update` (also observed interplay with `wm_page.link`)

## Full Input Parameters

```json
{"action": "update", "id": "wiki:tasks:6c372d", "status": "done"}
{"action": "update", "id": "wiki:tasks:6c372d", "status": "in-progress"}
```

## Full Error Output

```
MCP error: {"code":"INTERNAL_ERROR","message":"Invalid transition: todo → done. Allowed: in-progress, cancelled"}
```

Second call returned `{"id":"wiki:tasks:6c372d","status":"updated"}` but `get` immediately after returned `status: todo`.

## Counter-evidence

The task file exists on disk (`.wm/wiki/tasks/6c372d.md`) with valid frontmatter at HEAD. After the update+link sequence the file was corrupted on disk (verified with `cat`/`read`), and `wm_task.get`/`wm_task.list` returned NOT_FOUND / empty despite the file existing.

## Workaround

Write/edit the `.md` file directly under `.wm/wiki/tasks/` with the correct frontmatter (title, id, type, status, priority, tags, relates_to), then run `wm_index.rebuild` + `wm_index.embed` to resync the graph/index.

## Reproduction Steps

1. Create or pick a task with status `todo` (e.g. `wiki:tasks:6c372d`).
2. Call `wm_task.update` with `status: "done"` → observe `Invalid transition: todo → done` error.
3. Call `wm_task.update` with `status: "in-progress"` → returns success.
4. Call `wm_page.link` to add a `relates_to` edge on the task.
5. Call `wm_task.get` on the task → observe `NOT_FOUND`; read the `.md` file → observe mangled frontmatter (`{}` block, missing id/title/type/tags/priority).

## Related

- @wiki/rules/tool-reliability-bug-tracking
- @wiki/rules/check-wm-tool-health-before-work