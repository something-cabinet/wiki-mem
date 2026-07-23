---
title: "Pattern: Rust Binary Integration Test"
type: pattern
tags: [testing, integration, mcp, rust]
status: reviewed
confidence: high
relates_to:
  - {type: references, target: wiki:patterns:mcp-response-format}
  - {type: references, target: wiki:tasks:task-s2ff4x-mcp-e2e-integration-tests}
  - {type: references, target: wiki:tasks:task-7d3uvn-cli-e2e-integration-tests}
  - {type: references, target: wiki:tasks:task-g5nm08-full-workflow-e2e-test}
---

## When to use

When writing integration tests for Rust CLI/MCP applications where you need to spawn the actual binary as a child process. Use this instead of unit-testing internals when you need to verify the full binary behavior including CLI parsing, MCP protocol compliance, and side effects.

## How it works

1. **Derive the binary path** from the test binary location: `std::env::current_exe()` gives the test binary path (e.g., `target/debug/deps/wm_core-<hash>.exe`). Walk up from `deps/` to `debug/` to find the main binary (`wm-cli.exe`).
2. **Create an isolated temp project** directory with minimal config files (no need for `wm init` — just create `.wm/config.json` and wiki subdirs directly).
3. **For CLI tests**: spawn binary with args, capture stdout/stderr/exit code via `std::process::Command::output()`.
4. **For MCP tests**: spawn with piped stdin/stdout via `Stdio::piped()`, communicate JSON-RPC 2.0 requests/responses line by line.
5. **Cleanup**: use `tempfile::tempdir()` for automatic cleanup on drop.

## Example

See `wm-core/tests/helpers/mod.rs` for the `MCPClient` struct and `run_cli` function.
See Knowns' `tests/helpers_test.go` for the same pattern in Go.

## Limitations

- MCP requires bidirectional stdin/stdout — cannot use PowerShell pipes for testing. Use named pipes or child process stdio.
- Binary must be pre-built. The `TEST_BINARY` env var can override the path.
- Tests are slower than unit tests due to process spawning overhead.

## Source

@wiki/tasks/s2ff4x @wiki/tasks/7d3uvn @wiki/tasks/g5nm08
