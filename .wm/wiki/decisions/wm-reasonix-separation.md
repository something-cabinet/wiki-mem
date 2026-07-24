---
id: wiki:decisions:wm-reasonix-separation
title: WM and Reasonix orchestrator are separate concerns
type: decision
status: approved
tags: [decision, approved, reasonix, orchestrator]
relates_to:
  - {type: references, target: wiki:specs:reasonix-wm-shim}
  - {type: references, target: wiki:patterns:compatibility-shim-pattern}
---
id: wiki:decisions:wm-reasonix-separation

## Context

During the Reasonix orchestrator integration, there was ambiguity about where the orchestrator (specialist subagent skills, ORCHESTRATOR.md, global Reasonix config) should live — inside the WM project's `wm init` flow, or as a separate standalone tool.

## Decision

WM and the Reasonix orchestrator are separate projects with a narrow integration point:

- **WM** (`apps/wm-core`, `apps/wm-cli`) is a knowledge engine with MCP tools, CLI, TUI, and web UI. Its `wm init` generates compatibility shims (CLAUDE.md, OPENCODE.md, GEMINI.md, REASONIX.md) that all redirect to WIKI-MEM.md. It does NOT install orchestrator skills, global config, or ORCHESTRATOR.md.

- **reasonix-orchestrate** (separate repo, `gitea.gehenna.work/vpp/reasonix-config`) is a standalone Rust binary that installs the full orchestrator system — 7 specialist subagent skills, ORCHESTRATOR.md with lane definitions, and global Reasonix config.toml updates.

The integration point: `wm init --platform reasonix` and `wm setup reasonix` generate a REASONIX.md shim that follows the same pattern as OPENCODE.md. Everything else is the orchestrator binary's job.

## Rationale

- Single responsibility: WM should not know about Reasonix subagent skills or their config
- Separate release cycles: the orchestrator can evolve independently of WM
- User choice: not every WM project needs Reasonix orchestrator
- Consistency: all runtimes get the same treatment — a thin shim pointing to WIKI-MEM.md

## Consequences

- WM's init code stayed small (~15 lines for the reasonix shim)
- The orchestrator is a standalone binary users opt into
- Adding future runtime support is just another shim in sync_agent_files()

## Related
- @wiki/specs/reasonix-wm-shim
- @wiki/patterns/compatibility-shim-pattern