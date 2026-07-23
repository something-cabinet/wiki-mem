---
title: "UPDATED: Skill directories are per-platform, not shared"
type: memory
tags: [skills, convention, path]
created_at: "2026-07-09T08:18:32.462Z"
updated_at: "2026-07-20T05:30:00.000Z"
supersedes: "@wiki/learnings/per-platform-skill-directories"
relates_to:
  - {type: references, target: wiki:learnings:per-platform-skill-directories}
---

## Original (outdated)

WM skills should be synced to `.agent/skills/` (singular, not `.agents/`). This matched Knowns convention but assumed a single shared directory worked for all platforms.

## Update (2026-07-20)

`.agent/skills/` is **not** a universal standard. Each AI coding platform has its own native skill directory, and `wm setup <platform>` must sync to the correct per-platform path:

| Platform | Native Skill Dir |
|----------|-----------------|
| OpenCode | `.opencode/skills/` |
| Codex | `.codex/skills/` |
| Antigravity | `.agents/skills/` |
| Claude | `.claude/skills/` |
| Kiro | `.kiro/skills/` |
| Cursor | `.cursor/skills/` |
| Generic agents | `.agent/skills/` (fallback) |

See `@wiki/learnings/per-platform-skill-directories` for the full finding.