---
{}
relates_to:
  - {type: references, target: wiki:tasks:refactor-wm-cli-mcp-to-register-handlers-directly}
---

---
title: Decision: CLI Commands Run Directly, Never Proxy Through HTTP
type: decision
status: approved
---

## Context

The `wm-cli` binary was refactored to proxy all page/graph/task operations through HTTP to a running `wm-server` daemon at `localhost:4090`. This meant CLI commands didn't work without the server running, tests couldn't run independently, and the CLI lost its identity as a standalone tool.

## Decision

The CLI must never proxy operations through HTTP. All commands run directly in-process using `create_engine()` + `wm_core::page::*` / `wm_core::graph::*` / `wm_core::task::*` APIs. The `http_call` function was removed entirely along with the `ureq` dependency.

## Rationale

- **Standalone operation:** CLI should work offline without a running server
- **Testability:** Integration tests spawn the CLI binary and expect it to work independently
- **Latency:** In-process execution is faster than HTTP round-trips (2.5s → 0.07s for page create)
- **Architecture clarity:** The HTTP daemon is for web UI and remote access; the CLI is for local direct use
- **No shared state needed:** Each CLI command creates a fresh engine, operates on local files, exits

## Consequences

- CLI tests (35) now pass instead of 14 failures
- No `ureq` dependency needed
- The `wm-server` HTTP daemon remains for web UI and long-running server scenarios
- One-shot CLI mode works without file watcher — inline `handle_file_change` handles graph updates

## Related
- @wiki/tasks/p0-wire-body-wiki-references-into-graph-builder
- @wiki/decisions/mcp-direct-handlers-over-proxy