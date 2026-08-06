---
title: "P2 polish: agents sync, platform tests, spec update, Gemini"
type: task
status: done
tags: [p2, platform, setup, tests]
priority: low
id: 0xskfm
acceptance_criteria:
  - text: "wm agents sync regenerates all platform compat entrypoints in one shot"
  - text: "Regression tests confirm wm setup codex produces valid TOML and wm setup opencode produces valid JSON with correct structure"
  - text: "Spec D1 in specs/wm-init-platform-agent-instruction-files-mcp-config is aligned with the actual architecture, and Gemini CLI is added to wm setup"
---

# P2 polish: agents sync, platform tests, spec update, Gemini

> *Imported from Knowns task `0xskfm`*

# P2 polish: agents sync, platform tests, spec update, Gemini

## Description


Remaining low-priority items from the platform setup parity work:

1. **`wm agents sync` command** — regenerate all compat entrypoints in one shot (like `knowns agents --sync`)
2. **Regression tests for platform setup** — test that `wm setup codex` produces valid TOML, `wm setup opencode` produces valid JSON with correct structure
3. **Spec update** — align D1 in `specs/wm-init-platform-agent-instruction-files-mcp-config` with actual architecture (setup is separate from init)
4. **Gemini CLI platform** — add to `wm setup` (informative, since Gemini manages its own config)


## Acceptance Criteria
