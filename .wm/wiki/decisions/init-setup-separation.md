---
{}
relates_to:
  - {type: implements, target: wiki:patterns:platform-aware-mcp-config}
---

id: wiki:decisions:init-setup-separation

## Context

Initially `wm init --platform` both generated agent compat entrypoints (CLAUDE.md, OPENCODE.md) AND MCP server config files (opencode.json, .kiro/settings/mcp.json). This conflated two concerns: agent guidance and platform integration.

Knowns separates these into `knowns init` (creates project + lightweight shims) and `knowns setup <platform>` (generates MCP config).

## Chosen approach

`wm init --platform` generates only compat entrypoints. `wm setup <platform>` handles MCP config with `--global` support. Skills are synced during setup.

## Alternatives considered

Combined approach was simpler to implement but harder to reason about, especially with `--global` flag semantics.

## Outcome

Clean separation of concerns. 6 platforms supported: claude, codex, opencode, kiro, cursor, antigravity.

## Source

@wiki/tasks/omuamh @wiki/tasks/wkm5xh