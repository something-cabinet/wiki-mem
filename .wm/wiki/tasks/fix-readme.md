---
title: Fix README
type: task
id: wiki:tasks:fix-readme
status: done
priority: medium
acceptance_criteria:
- text: README.md install section uses the correct published package name or accurate source-build instructions
- text: CLI Commands table verified against `wm --help` output — no stale or missing commands
- text: Platform setup list matches actual `wm setup` targets
- text: MCP tool groups match current ToolRegistry surface
- text: Architecture section reflects wm-server single HTTP daemon + Angular frontend
- text: README.md renders correctly as markdown (no broken code blocks or links)
time_started: 2026-08-06T02:49:18.571831+00:00
relates_to:
- type: implements
  target: wiki:specs:fix-readme
time_spent: 0h 8m
---

Fix and update the repository root README.md to be accurate and current with the actual wm tooling.

Known issues to address:
- Install command references a placeholder npm scope (`npm install -g @something-cabinet/wm-cli`) — verify the real published package name (@scope/wm-cli) or correct the install instructions.
- CLI command table should match the actual `wm-cli` command surface (check `wm --help` / clap definitions; table currently omits some commands and may list stale ones).
- Verify `wm setup <platform>` platform list (README mentions Claude, OpenCode, Kiro, Gemini, Copilot; config example shows codex too).
- Verify MCP tool groups table matches the current ToolRegistry surface.
- Confirm architecture section reflects single HTTP daemon deployment (wm-server on :4090) rather than implying CLI-only.
- Check README frontmatter/conventions consistency with wiki core README page (wiki:core:README).

Out of scope: wiki:reference:README (that page documents upstream Knowns and is separately tracked).