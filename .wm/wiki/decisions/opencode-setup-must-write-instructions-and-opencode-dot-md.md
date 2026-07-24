---
id: wiki:decisions:opencode-setup-must-write-instructions-and-opencode-dot-md
title: "Decision: wm setup opencode must write instructions + OPENCODE.md"
type: decision
status: approved
tags: [decision, opencode, setup, instructions, shim]
relates_to:
  - {type: references, target: wiki:patterns:compatibility-shim-pattern}
  - {type: references, target: wiki:patterns:embed-shim-templates}
---
id: wiki:decisions:opencode-setup-must-write-instructions-and-opencode-dot-md

## Context

OpenCode V2 auto-detects `AGENTS.md` from the project root by scanning upward from the current directory. But OpenCode also supports an `instructions` field in `opencode.json` that points to additional instruction files. The project wanted `opencode.json` to point to `OPENCODE.md` as the entrypoint, which then directs agents to read `WIKI-MEM.md`.

Previously, `wm setup opencode` only wrote:
- `opencode.json` with the `mcp` config (no `instructions` field)
- `AGENTS.md` from the embedded shim templates (for auto-detection)
- Skills to `.opencode/skills/`

It did NOT write `OPENCODE.md` or include `instructions` in `opencode.json`.

## Decision

The `wm setup opencode` command MUST:
1. Write `instructions: ["OPENCODE.md"]` in `opencode.json` so OpenCode V2's future `instructions` resolution loads it
2. Write `OPENCODE.md` from the embedded shim template alongside `AGENTS.md`
3. `sync_agent_files()` must also write `OPENCODE.md` when the `opencode` platform is in the target list

Additionally, `write_merged_json()` must preserve the `instructions` field from new configs when merging into existing configs.

## Rationale

- `OPENCODE.md` is a thin shim whose first directive says "Start with `wm_initial`" and "read WIKI-MEM.md" — this is the proper entrypoint for OpenCode
- `AGENTS.md` remains for auto-detection (OpenCode V2 scans for it), but the `instructions` field makes the chain explicit
- Without this, agents starting via OpenCode would miss the bootstrap flow
- The `write_merged_json` change ensures `instructions` survives re-runs of `wm setup opencode`

## Consequences

- `wm setup opencode` now outputs both files: `OPENCODE.md` and `opencode.json`
- `wm setup all` also writes `OPENCODE.md`
- `write_merged_json` handles a third key (`instructions`) alongside `mcp` and `mcpServers`
- All existing `opencode.json` files missing `instructions` will get it added on next `wm setup opencode` run

## Related
- @wiki/patterns/compatibility-shim-pattern
- @wiki/patterns/embed-shim-templates
