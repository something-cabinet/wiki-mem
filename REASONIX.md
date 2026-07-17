# REASONIX

Compatibility entrypoint for runtimes that auto-detect `REASONIX.md`.

<!-- WIKI-MEM GUIDELINES START -->

**CRITICAL: You MUST read and follow `WIKI-MEM.md` in the repository root before doing any work. It is the canonical source of truth for all agent behavior in this project.**

## Canonical Guidance

- WM is the repository memory layer for humans and the AI-friendly working layer for agents.
- The source of truth for repo-level agent guidance is `WIKI-MEM.md`.
- Load behavior, memory policy, and workflow rules from `WIKI-MEM.md`; treat this file only as a compatibility entrypoint.
- If this file and `WIKI-MEM.md` differ, follow `WIKI-MEM.md`.

## Specialist Skills

Specialist subagent skills are installed under `.reasonix/skills/`:

- `fixer` — bounded implementation
- `designer` — UI/UX design
- `architect` — system design and trade-offs
- `code-reviewer` — general code review
- `rust-reviewer` — Rust-specific review
- `database-reviewer` — schema and query review
- `rust-build-resolver` — dependency and build issues

These are convention-based subagent prompts, not enforced permissions. Use them within the WM workflow via `run_skill()`.

## Quick Reference

```bash
wm-cli mcp              # Start MCP server
wm-cli search <q>       # Search the wiki
wm-cli page list        # List wiki pages
wm-cli lint check       # Check wiki health
```

<!-- WIKI-MEM GUIDELINES END -->
