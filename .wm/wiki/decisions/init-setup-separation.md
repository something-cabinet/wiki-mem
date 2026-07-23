---
title: "Decision: Separate init and setup commands"
type: decision
tags: [cli, platform, architecture]
status: reviewed
confidence: high
decision:
  context: |
    Initially `wm init --platform` both generated agent compat entrypoints (CLAUDE.md, OPENCODE.md) AND MCP server config files (opencode.json, .kiro/settings/mcp.json). Knowns separates these into `knowns init` (project + shims) and `knowns setup <platform>` (MCP config).
  options:
    - "Combine in wm init --platform (generates everything in one step)"
    - "Split into wm init (shims) + wm setup <platform> (MCP config)"
  rationale: |
    Following Knowns' proven pattern avoids confusion. Users who just want CLAUDE.md can run `wm init --platform claude`. Users who want MCP integration run `wm setup claude`. The separation also makes `--global` clearer — MCP configs can be installed at user level while shims stay project-local.
  outcome: |
    `wm init --platform` now generates only thin compat entrypoints. `wm setup <platform>` generates MCP config files per platform's convention. `wm setup <platform> --global` writes to user-level config paths. Skills are synced to per-platform dirs (`.claude/skills/`, `.kiro/skills/`) during setup.
relates_to:
  - {type: implements, target: wiki:patterns:platform-aware-mcp-config}
  - {type: references, target: wiki:tasks:task-omuamh-wm-init-parity-with-knowns-init}
  - {type: references, target: wiki:tasks:task-wkm5xh-research-platform-configskill-dirs-from-knowns-source-validate-wm-parity}
---

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
