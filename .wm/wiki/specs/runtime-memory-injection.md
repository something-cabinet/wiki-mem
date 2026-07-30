---
title: Runtime Memory Injection via MCP Server
type: spec
tags:
- spec
- mcp
- hooks
- runtime-memory
status: approved
relates_to:
- type: references
  target: wiki:core:ARCHITECTURE
---

## Overview

Replace `.kiro/hooks/wm-hooks.json` with MCP server-side runtime memory injection. The `wm-cli` MCP server automatically injects project context (version, core pages, active tasks) on the first tool call of each session, for all platforms that connect via MCP.

## Locked Decisions

- D1: Use MCP server injection — no hook files on disk
- D2: Inject context for all MCP platforms (opencode, claude, kiro, codex, cursor, antigravity)
- D3: Auto mode — inject on first MCP call of each session

## Requirements

### Functional Requirements
- FR-1: MCP server must detect first tool call of a session
- FR-2: On first call, inject context block containing: wm-cli version, list of core pages, count of active tasks
- FR-3: Context injection must not interfere with the tool response
- FR-4: Only inject once per session — subsequent calls skip injection
- FR-5: Remove `.kiro/hooks/wm-hooks.json` generation from the Kiro setup

### Non-Functional Requirements
- NFR-1: Zero overhead on subsequent calls (no check overhead)
- NFR-2: Context format should be concise (< 500 bytes)
- NFR-3: Must not break any existing MCP tool responses

## Acceptance Criteria

- [ ] AC-1: First MCP tool call in a session returns context block + normal tool response
- [ ] AC-2: Second+ MCP tool calls in same session return only normal tool response
- [ ] AC-3: Context block contains wm-cli version
- [ ] AC-4: Context block lists core pages or indicates none exist
- [ ] AC-5: Kiro setup no longer creates `.kiro/hooks/wm-hooks.json`
- [ ] AC-6: All existing MCP tools continue working

## Scenarios

### Scenario 1: Fresh Session (auto inject)
**Given** a new Kiro/Claude/OpenCode session connects to wm-cli MCP
**When** the first tool call arrives
**Then** MCP server responds with context block followed by the tool's normal response
**And** subsequent tool calls return only normal responses

### Scenario 2: No Project
**Given** wm-cli MCP is running outside a wiki project
**When** the first tool call arrives
**Then** MCP server injects "no wiki project found" context
**And** continues normally

### Scenario 3: Kiro Without Hook File
**Given** user runs `wm init --platform kiro`
**When** setup completes
**Then** `.kiro/hooks/wm-hooks.json` is NOT created
**But** `.kiro/settings/mcp.json` still exists with the MCP server config

## Technical Notes

- The `rmcp` server handles incoming tool calls — track session state there
- Use a `HashMap<SessionId, bool>` to track whether first call has been served
- Context format:
  ```
  [Wiki Memory Engine v0.2.2]
  Core pages: ARCHITECTURE, CONVENTIONS, README (3)
  Active tasks: 2
  ```
- Remove the hooks directory and files from `EmbeddedFiles`
- Remove the hooks writing code from `setup_platform_mcp()` in main.rs

## Open Questions

- [ ] Should the context include a brief how-to (`wm search`, `wm page get`)?
- [ ] Should we add a flag to disable injection?