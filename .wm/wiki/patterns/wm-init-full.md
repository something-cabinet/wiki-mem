---
title: Pattern: wm init --full — Chain System + Project Setup
type: pattern
tags: [pattern, setup, bootstrap, knowns]
status: reviewed
relates_to:
  - {type: references, target: wiki:decisions:wm-self-upgrade}
  - {type: references, target: wiki:specs:wm-self-install}
---

---
id: wiki:patterns:wm-init-full
title: "Pattern: wm init --full — Chain System + Project Setup"
type: pattern
status: reviewed
tags: [pattern, setup, bootstrap, knowns]
relates_to:
  - {type: references, target: wiki:decisions:wm-self-upgrade}
  - {type: references, target: wiki:specs:wm-self-install}
  - {type: references, target: wiki:decisions:init-setup-separation}
---
id: wiki:patterns:wm-init-full

## Problem

New developers or fresh clones need multiple steps to start working: install the binary, configure MCP, initialize the project. Each step requires a different command with different scopes (system vs project).

## Solution

A single `wm init --full` command that chains three operations in dependency order:

1. `wm upgrade` — copies binary to `~/.wm/bin/`, registers on PATH
2. `wm init` — creates `.wm/wiki/` project structure (project root must exist first so later steps can resolve via `detect_project_root`)
3. `wm setup opencode` (equivalent) — generates `opencode.json` with `["wm-cli", "mcp"]` (canonical PATH-based command, safe because the binary was just installed), writes `OPENCODE.md`, and syncs skills to `.opencode/skills/`

The opencode.json uses `["wm-cli", "mcp"]` as the canonical command rather than a resolved binary path. This is safe because `--full` installs the binary before writing the config. For non-standard installations, use `wm setup opencode` which resolves the actual binary path.

## When to Use

- Setting up WM on a new machine
- Bootstrapping a fresh clone of a WM project
- First-time user onboarding — single command, complete setup

## When Not to Use

- Existing projects that already have MCP config (use `wm setup opencode` to update)
- CI/CD environments where binary installation is handled separately
- Users who want a specific binary path in the config (use `wm setup opencode`)

## Related

- @wiki/decisions/wm-self-upgrade
- @wiki/specs/wm-self-install
- @wiki/decisions/init-setup-separation
