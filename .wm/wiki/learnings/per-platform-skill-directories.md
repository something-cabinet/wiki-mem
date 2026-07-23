---
title: Each AI Platform Has Its Own Native Skill Directory
type: concept
status: draft
tags: [skills, platform, setup, learning]
created_at: "2026-07-20"
relates_to:
  - {type: references, target: wiki:memory:67fpsz}
  - {type: references, target: wiki:howto:platform-setup}
  - {type: references, target: wiki:specs:wm-sdd-skills}
  - {type: references, target: wiki:tasks:task-n7oz3d}
  - {type: references, target: wiki:tasks:task-wkm5xh}
---

## Finding

The `.agent/skills/` directory (singular) is **not** a universal standard that all AI coding platforms recognize. Each platform has its own native skill discovery path:

| Platform | Native Skill Directory | Notes |
|----------|----------------------|-------|
| OpenCode | `.opencode/skills/` | Auto-scanned on startup |
| Codex    | `.codex/skills/` | Also scans `.agents/skills/` |
| Antigravity | `.agents/skills/` | `.agent/skills/` works as legacy fallback |
| Claude Code | `.claude/skills/` | Dedicated per-platform dir |
| Kiro     | `.kiro/skills/` | Dedicated per-platform dir |
| Cursor   | `.cursor/skills/` | — |
| Gemini CLI | platform-managed | Uses Gemini's own config |

## Background

The WM project originally mapped all non-Claude, non-Kiro platforms to a shared `.agents/skills/` directory (based on Knowns convention). This was later changed to `.agent/skills/` (singular) in code, but the docs still referenced `.agents/skills/` (plural). Neither is correct for platforms like OpenCode and Codex that have their own native skill directories.

## What Was Fixed

In `wm-cli/src/main.rs`, the `wm setup` command was changed to sync skills to each platform's **native** directory instead of the shared `.agent/skills/`:

- `opencode` → `.opencode/skills/` (was `.agent/skills/`)
- `codex` → `.codex/skills/` (was `.agent/skills/`)
- `antigravity` → `.agents/skills/` (was `.agent/skills/`)

The `all` handler also had `.opencode/skills/`, `.codex/skills/`, and `.agents/skills/` added to its sync targets.

## Related Pages

- `@wiki/memory/67fpsz` — Original decision to use .agent/skills/ (now outdated)
- `@wiki/howto/platform-setup` — Platform setup documentation (needs update)
- `@wiki/specs/wm-sdd-skills` — SDD skills spec with D4 mapping (needs update)
- `@wiki/tasks/task-n7oz3d` — Previous sync fix
- `@wiki/tasks/task-wkm5xh` — Research platform config/skill dirs

## Outdated References That Need Fixing

The following documentation still references the old shared-directory model and should be updated:

- `.wm/wiki/howto/platform-setup.md` — Skill Dir column shows `.agents/skills/` for OpenCode, Codex, Antigravity
- `.wm/wiki/specs/wm-sdd-skills.md` — D4 mapping groups opencode, codex, antigravity, cursor, gemini under `.agents/skills/`
- `.wm/wiki/memory/67fpsz.md` — Declares `.agent/skills/` as convention, which is now partially wrong
