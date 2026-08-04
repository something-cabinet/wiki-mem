---
title: init setup separation
id: wiki:decisions:init-setup-separation
type: decision
implementation_notes: 'UPDATE 2026-07-31: path-resolution subsection is stale. Per @wiki/specs/remove-self-install-flow, the two-tier resolution (is_installed() → ~/.wm/bin check, current_exe() fallback) and wm init --full are removed. MCP config generation now always writes ["wm-cli", "mcp"]. The init/setup separation itself remains valid.'
relates_to:
  - {type: implements, target: wiki:patterns:platform-aware-mcp-config}
---

## Context

Initially `wm init --platform` both generated agent compat entrypoints (CLAUDE.md, OPENCODE.md) AND MCP server config files (opencode.json, .kiro/settings/mcp.json). This conflated two concerns: agent guidance and platform integration.

Knowns separates these into `knowns init` (creates project + lightweight shims) and `knowns setup <platform>` (generates MCP config).

## Chosen approach

`wm init --platform` generates only compat entrypoints. `wm setup <platform>` handles MCP config with `--global` support. Skills are synced during setup.

### Path resolution strategy

Two-tier path resolution for MCP config command values:

- **`wm init --full`** — writes canonical `["wm-cli", "mcp"]` (assumes binary is on PATH). This is safe because `--full` installs the binary to `~/.wm/bin/` and registers it on PATH before writing the config.
- **`wm setup opencode`** — resolves the actual binary path. If `is_installed()` returns true (binary exists at `~/.wm/bin/wm-cli`), uses `["wm-cli", "mcp"]`. Otherwise, uses `std::env::current_exe()` to get the absolute path to the running binary.

This means:
- `wm init --full` always produces a clean, portable `opencode.json` with `wm-cli`
- `wm setup opencode` adapts to the environment — if the user is running from a debug build, it writes the full debug path; if installed globally, it writes `wm-cli`
- `wm init --platform` does NOT generate MCP config at all (respects the separation)

## Alternatives considered

Combined approach was simpler to implement but harder to reason about, especially with `--global` flag semantics.

## Outcome

Clean separation of concerns. 6 platforms supported: claude, codex, opencode, kiro, cursor, antigravity.

## Source

@wiki/tasks/omuamh @wiki/tasks/wkm5xh @wiki/tasks/review-wm-init--opencodejson-not-generated-during-init
