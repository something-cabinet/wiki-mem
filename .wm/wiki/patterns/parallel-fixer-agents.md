---
id: wiki:patterns:parallel-fixer-agents
title: "Pattern: Parallel Fixer Agents for Batch File Editing"
type: pattern
tags: [pattern, workflow, delegation, batch]
status: draft
relates_to:
  - {type: references, target: wiki:patterns:task-subagents-for-delegation}
---
id: wiki:patterns:parallel-fixer-agents

## Problem

Making the same structural change across many files (e.g., removing comments from 90 source files, adding a section to 8+ skill files, renaming a symbol across modules) is slow when done sequentially — each file requires reading, editing, and context retention.

## Solution

Spawn parallel `fixer` subagents each handling a scoped batch of files (2–8 files per agent). Give each fixer the exact transformation rules and per-file customizations. The orchestrator tracks all agents, reconciles results, and runs a final verification pass.

```
Orchestrator
  ├── task("Fix group A: main.rs, tui.rs, ...") → fixer
  ├── task("Fix group B: template/mod.rs, task/mod.rs, ...") → fixer
  ├── task("Fix group C: engine/, graph/, search/ ...") → fixer
  └── waits → reconciles → verifies
```

### Key success factors

1. **Scoped per agent** — Each agent gets a bounded file list (by module directory). Avoid overlapping write scopes.
2. **Explicit transformation rules** — Tell each fixer exactly which comment types to remove, which to extract, which to rename. Don't assume they'll infer.
3. **Self-documenting naming** — Instruct fixers to rename functions instead of keeping doc comments (`terminal_supports_unicode()` over `/// Returns true if`).
4. **Second pass for stragglers** — After all agents complete, run a comprehensive scan and dispatch a final fixer for missed files. ~20% of files will be missed on the first pass if scope wasn't exhaustive.

## When to Use

- Batch operations affecting 10+ files with similar transformation patterns
- Comment removal, template updates, symbol renames, import path fixes
- Any change that follows a consistent rule but needs per-file judgment

## When Not to Use

- A single small change (<20 lines, one file)
- Changes requiring deep architectural understanding (use oracle review)
- Edits with overlapping write scopes that conflict

## Related
- @wiki/tasks/c19d50
- @wiki/patterns/task-subagents-for-delegation
- @wiki/learnings/session-skills-alignment-mcp-tools
