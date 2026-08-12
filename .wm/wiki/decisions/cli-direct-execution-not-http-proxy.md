---
title: 'Decision: CLI Commands Run Directly, Never Proxy Through HTTP'
type: decision
id: wiki:decisions:cli-direct-execution-not-http-proxy
status: active
relates_to:
  - {type: references, target: wiki:tasks:42b32a}
---
id: wiki:decisions:cli-direct-execution-not-http-proxy

---
id: wiki:decisions:cli-direct-execution-not-http-proxy
title: Decision: CLI Commands Run Directly, Never Proxy Through HTTP
type: decision
status: approved
---
id: wiki:decisions:cli-direct-execution-not-http-proxy

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

## Implementation (2026-08-12)

Reinstated and implemented by @wiki/tasks/cli-mcp-in-process-refactor: `http_client.rs` (231 lines) and the `ureq` dependency deleted from wm-cli; all ~29 call sites dispatch in-process via `create_engine()` + the tool registry. The HTTP-proxy reversal is superseded by this implementation.

## Related
- @wiki/tasks/fbe6a0
- @wiki/tasks/cli-mcp-in-process-refactor
- @wiki/decisions/mcp-direct-handlers-over-proxy