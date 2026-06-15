---
id: r8n30s
title: Foundation + MCP Transport
status: done
priority: high
labels:
  - from-spec
  - go-mode
  - foundation
createdAt: '2026-06-15T11:31:04.833Z'
updatedAt: '2026-06-15T11:38:50.900Z'
timeSpent: 0
spec: specs/local-knowledge-engine-rust
fulfills:
  - AC-1
---
# Foundation + MCP Transport

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Project skeleton, crate stack, directory layout, core data structures, MCP JSON-RPC transport, tool argument helpers, structured errors, project auto-detection, config loading, panic recovery, signal handling, escaped newlines, utility helpers. Spec: specs/local-knowledge-engine-rust
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Cargo workspace compiles (cargo build)
- [x] #2 MCP transport reads JSON-RPC from stdin, writes to stdout, errors to stderr
- [x] #3 Project auto-detection walks up from cwd for .wm/config.json
- [x] #4 ToolArgs helpers: require_string, optional_text (with unescape), optional_int, optional_bool, optional_string_array
- [x] #5 ToolError: codes REQUIRED_FIELD, NOT_FOUND, NO_PROJECT, INVALID_ACTION with hints
- [x] #6 catch_unwind around every handler returns structured error instead of crash
- [x] #7 SIGINT/SIGTERM triggers flush + persist + clean exit
- [ ] #8 Rotating file logger writes to ~/.wm/logs/wm.log
- [x] #9 Utility helpers: unescape_text, truncate_str, slugify, first_non_empty
- [ ] #10 Audit logging bounded(1024) channel with overflow counting
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Milestone 1: Built workspace with wm-core and wm-cli crates. Core data structures (EngineState, EdgeType, PageType, WikiPageMeta, SourceEntry, AuditEvent). MCP JSON-RPC transport with initialize/tools/list/tools/call dispatch. ToolArgs with require_string, optional_text (unescape), optional_int, optional_bool, optional_string_array. ToolError with codes REQUIRED_FIELD, NOT_FOUND, NO_PROJECT, INVALID_ACTION, INTERNAL_ERROR + hints. catch_unwind panic recovery in transport. Signal handling via tokio::signal::ctrl_c. Utility helpers: unescape_text, truncate_str, slugify, first_non_empty, format_duration, contains_str (all tested). Project auto-detection walks up from cwd or WM_PROJECT env var. Config loading via config.json.
<!-- SECTION:NOTES:END -->

