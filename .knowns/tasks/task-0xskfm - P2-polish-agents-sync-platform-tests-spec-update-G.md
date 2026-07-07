---
id: 0xskfm
title: 'P2 polish: agents sync, platform tests, spec update, Gemini'
status: done
priority: low
labels:
  - p2
  - platform
  - setup
  - tests
createdAt: '2026-07-06T18:50:52.976Z'
updatedAt: '2026-07-07T06:19:52.158Z'
timeSpent: 0
---
# P2 polish: agents sync, platform tests, spec update, Gemini

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Remaining low-priority items from the platform setup parity work:

1. **`wm agents sync` command** — regenerate all compat entrypoints in one shot (like `knowns agents --sync`)
2. **Regression tests for platform setup** — test that `wm setup codex` produces valid TOML, `wm setup opencode` produces valid JSON with correct structure
3. **Spec update** — align D1 in `specs/wm-init-platform-agent-instruction-files-mcp-config` with actual architecture (setup is separate from init)
4. **Gemini CLI platform** — add to `wm setup` (informative, since Gemini manages its own config)
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

