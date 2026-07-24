---
id: wiki:rules:tool-reliability-bug-tracking
title: Report Wiki Tool Reliability Issues as Tasks
type: rule
status: active
tags:
  - rule
  - workflow
  - bug-tracking
---
id: wiki:rules:tool-reliability-bug-tracking

Whenever a wiki MCP tool (`wm_page.*`, `wm_index.*`, `wm_search.*`, `wm_*`) behaves unreliably — returns errors that don't match reality, fails on valid inputs, has undocumented parameters, or otherwise impedes autonomous agent workflows — **a task must be created immediately** in `.wm/wiki/tasks/` with full reproduction details.

## Rationale

The wiki MCP tools are the primary interface for agents to read and update project knowledge. If they break or behave unpredictably, the entire autonomous workflow degrades. These bugs are invisible to humans (who use git/editor directly) but catastrophic for AI agents. Every incident must be captured.

## Required Fields in the Bug Task

Each task must include:

1. **Title** — `Wiki Tool Reliability: {tool_name} — {short description}`
2. **Bug Description** — What happened vs what was expected
3. **Tool Name** — Exact tool used (e.g. `wm-dev_wm_page`)
4. **Full Input Parameters** — What was passed (redact nothing, even if it feels obvious)
5. **Full Error Output** — The exact error message returned
6. **Counter-evidence** — What proves the error is wrong (e.g. "the page exists, `get` works with the same ID")
7. **Workaround** — How to achieve the same result without the broken tool (e.g. write file directly)
8. **Reproduction Steps** — From-scratch steps someone else can follow

## Anatomy of a Bad Interaction

A tool is considered "unreliable" when any of these happen:

- Returns "not found" for something that exists (`get` finds it but `update` doesn't)
- Rejects parameters without documentation (e.g. requires `path` but doesn't list it)
- Accepts a parameter name different from what's documented (`page_id` vs `id`)
- Has required parameters that aren't shown in the tool signature
- Has inconsistent behavior between sub-actions (one action works, another doesn't)

## Mandatory Trigger: File Directly Instead

When a tool fails, **do not retry with different parameter combinations** more than twice. Instead:
1. Write/edit the `.md` file directly in `.wm/wiki/` using the standard file tools
2. Run `wm_index` rebuild to sync the wiki index
3. Create the bug task documenting the failure

## After Creating the Bug Task

- Link the bug task from the new task you were trying to create (the reason you hit the bug)
- The bug task becomes input for the next wiki-infrastructure improvement sprint
