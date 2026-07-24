# OPENCODE

Compatibility entrypoint for runtimes that auto-detect `OPENCODE.md`.

**CRITICAL: Start with `wm_initial` via MCP when available. Use `wm_help` for tool schemas and workflow routing on demand. You MUST read and follow `WIKI-MEM.md` in the repository root — it is the canonical source of truth.**

<!-- WIKI-MEM GUIDELINES START -->

## Runtime Guidance

- WM is the repository memory layer for humans and the AI-friendly working layer for agents.
- `WIKI-MEM.md` is the canonical repo-level guidance file. Read it before doing any work.
- If this file and `WIKI-MEM.md` differ, follow `WIKI-MEM.md`.
- MCP `wm_initial` is the primary AI bootstrap: project state, tool domains, and active rules.
- MCP `wm_help` provides on-demand tool schemas and descriptions.

## Minimum Rules

- Use WM MCP tools (`wm_*`) as the canonical system for tasks, docs, templates, memory, search, code intelligence, and workflow state.
- Never manually edit WM-managed task or doc markdown.
- Search first, then read only relevant docs and code.
- Plan before implementation unless the user explicitly overrides that workflow.
- Validate before considering work complete.
- Proactively capture durable memory; do not wait for explicit instruction.

## Quick Reference

```bash
wm-cli serve              # Start MCP server
wm init                   # Init project
wm search <q>             # Search wiki
wm task board             # Task board
wm lint check             # Wiki health
wm validate               # Validate refs
```

<!-- WIKI-MEM GUIDELINES END -->
