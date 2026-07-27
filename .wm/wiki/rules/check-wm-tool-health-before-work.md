---
title: Check WM Tool Health Before Starting Work
type: rule
id: wiki:rules:check-wm-tool-health-before-work
status: active
tags: [rule, tool-reliability, workflow, health]
---

## Rule: Check WM Tool Health Before Starting Work

Before beginning any implementation, research, or documentation task, check whether the WM tools (MCP and CLI) are behaving correctly. If they misbehave, create a bug task before proceeding with the original work.

### Why

WM MCP tools (`wm_page.*`, `wm_index.*`, `wm_search.*`, `wm_*`) are the primary interface for agents to read and update project knowledge. If they break or behave unpredictably, the entire autonomous workflow degrades. These bugs are invisible to humans (who use git/editor directly) but catastrophic for AI agents. See `@wiki/rules/tool-reliability-bug-tracking`.

### Check Procedure

1. **Quick smoke test** — Call `wm_project.status` or `wm_initial` and verify it returns reasonable state (node count > 0, no unexpected errors).

2. **Index health** — Call `wm_index.status` and check:
   - `stale` is `false` — if stale, run `wm_index.rebuild` first
   - Sections and vectors are indexed (> 0)
   - The model is loaded if embeddings are expected

3. **Graph integrity** — Call `wm_graph.stats` and check:
   - Total nodes and edges look reasonable
   - Key page types are present (core, task, spec, rule, etc.)
   - No suspicious type counts (e.g., 0 rules when 8 rule files exist)

4. **Page read smoke test** — Call `wm_page.get` on a known page and verify it returns content.

5. **Search smoke test** — Call `wm_search.query` on a known term and verify results.

6. **Create task if broken** — If any check fails (returns wrong data, errors on valid input, silent incorrect results):
   - Create a `wiki:tasks:tool-reliability-{slug}` task with full reproduction details
   - Follow the bug task format in `@wiki/rules/tool-reliability-bug-tracking`
   - If the tool is completely broken, work around it using direct file operations
   - Then proceed with the original work

### Exceptions

- Quick checks only — spend at most 30s on the full check suite
- If the same tool was confirmed working in the same session, skip re-checking it
- If a known bug already has a task, don't duplicate — add a note to the existing task

### Related

- `@wiki/rules/tool-reliability-bug-tracking` — Bug task format and filing requirements
- `@wiki/rules/no-warnings` — Tool warnings are also defects
- `@wiki/patterns:critical-patterns` — Known MCP tool pitfalls