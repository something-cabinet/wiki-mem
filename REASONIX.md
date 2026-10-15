# REASONIX

Compatibility entrypoint for runtimes that auto-detect `REASONIX.md`.

<!-- WIKI-MEM GUIDELINES START -->

**CRITICAL: You MUST read and follow `WIKI-MEM.md` in the repository root before doing any work. It is the canonical source of truth for all agent behavior in this project.**

## Canonical Guidance

- WM is the repository memory layer for humans and the AI-friendly working layer for agents.
- The source of truth for repo-level agent guidance is `WIKI-MEM.md`.
- Read `WIKI-MEM.md` first whenever the runtime supports reading repository files.
- Load behavior, memory policy, and workflow rules from `WIKI-MEM.md`; treat this file only as a compatibility entrypoint.
- If this file and `WIKI-MEM.md` differ, follow `WIKI-MEM.md`.

## Minimum Rules

- Use WM MCP tools (`wm_*`) as the canonical system for tasks, docs, templates, memory, search, code intelligence, and workflow state.
- Never manually edit WM-managed task or doc markdown.
- Search first, then read only relevant docs and code.
- Use `wm_search.query` for discovery; use `wm_search.retrieve` when a workflow needs structured context with citations.
- For code operations, use `wm_code.search` for AST-aware search, symbol lookup, and dependency analysis.
- Plan before implementation unless the user explicitly overrides that workflow.
- Validate before considering work complete.
- Use memory tools: `wm_memory.list` at session start, `wm_memory.add` after tasks for reusable knowledge.
- Proactively capture durable memory; do not wait for explicit instruction.
- Read all rules from `.wm/wiki/rules/` at session start and obey them.

## Quick Reference

```bash
wm-cli init                     # Initialize wiki structure in this project
wm-cli mcp                      # Start the MCP server (stdio)
wm-cli setup <platform>         # Write that platform's MCP/agent config
wm-cli search query <query>     # Search the wiki
wm-cli task board               # Task board by status
wm-cli page list                # List wiki pages
wm-cli lint check               # Wiki health check
wm-cli validate                 # Validate refs + SDD coverage
wm-cli status                   # Engine and index status
```

<!-- WIKI-MEM GUIDELINES END -->
