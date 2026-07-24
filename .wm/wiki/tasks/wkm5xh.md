---
title: Research platform config/skill dirs from Knowns source — validate WM parity
type: task
status: done
tags: [research, platform, knowns, config, skills]
priority: high
id: wkm5xh
---

# Research platform config/skill dirs from Knowns source — validate WM parity

> *Imported from Knowns task `wkm5xh`*

# Research platform config/skill dirs from Knowns source — validate WM parity

## Description


For each platform Knowns supports, find the exact config file path and skill directory location from the Knowns Go source code. Then double-check WM's implementation matches.

**Knowns platforms to research:**
- Claude Code (.mcp.json)
- Kiro (.kiro/settings/mcp.json)
- OpenCode (opencode.json)
- Codex (.codex/config.toml)
- Cursor (.cursor/mcp.json)
- Antigravity (~/.gemini/antigravity/mcp_config.json)
- Claude Desktop (global app config)

**Also research:**
- Where Knowns stores skills per platform (`.agents/skills/` vs platform-specific locations)
- How `knowns setup --global` resolves paths per platform
- Whether `knowns agents --sync` generates any platform-specific files beyond the root compat entrypoints

**Then validate WM:**
- Check wm-cli/src/main.rs for current `setup` and `init` platform paths
- Check wm-core/src/skill.rs for skill directory references
- Check wm-core/tests/helpers/mod.rs for test project setup
- Report any mismatches between Knowns' locations and WM's locations


## Acceptance Criteria



## Implementation Notes


## Oracle Review Summary

### P0 — Breaks platform usage
1. **Codex config format wrong** — WM writes `.mcp.json` (JSON), Codex expects `.codex/config.toml` (TOML with `[mcp_servers]`)
2. **Codex `--global` path wrong** — WM writes `~/.codex/.mcp.json`, Codex expects `~/.codex/config.toml`

### P1 — Wrong but partially works
3. **Claude/codex shared arm** — They diverge in format and path, must be split
4. **Antigravity missing** — Not in `wm setup` at all
5. **Claude `--global` path wrong** — WM writes `~/.mcp.json`, should write to Claude Desktop config
6. **Per-platform skills** — Knowns stores skills per-platform (`.claude/skills/`, `.kiro/skills/`)

### P2 — Defer
7. Gemini CLI — no-op since platform-managed
8. No `wm agents --sync` command
9. Error message outdated (missing antigravity)
10. Spec drift (D1 says "setup in init" but implemented separately)
11. No regression tests for platform setup

### Correct as-is
OpenCode, Kiro, Cursor paths ✅, compat shim pattern ✅, merge logic ✅, `.agents/skills/` ✅

## Completed 2026-07-07
- Added `wm agents --sync` command (regen all compat entrypoints) ✅
- Fixed spec drift (D1: init/setup separation) ✅
- Added 5 platform regression tests (opencode JSON, codex TOML, kiro JSON, cursor JSON, agents sync) ✅
