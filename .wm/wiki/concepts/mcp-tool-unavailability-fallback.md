---
title: "Failure: MCP Tool Unavailability — Manual Validation Fallback"
type: concept
status: reviewed
tags: [failure, mcp, fallback, validation, debugging]
---

## What went wrong

Repeatedly calling `wm_validate_check` for spec validation failed because the MCP server process was dead (`context canceled`). The call was retried **4 times** before switching to manual fallback — violating the project's own **tool-reliability-bug-tracking** rule which says: "do not retry more than twice, then file directly."

Additionally, no bug task was created for the MCP server reliability failure, which is required by the same rule.

## Root cause

The `mcpmon` process manager (which wraps the `wm` MCP server) died due to a Go-context cancellation — likely from:
- Idle timeout killing the child process
- Stdio pipe disconnect between the AI client and `mcpmon`
- Stale process state from a previous session

The WM binary itself was healthy (passed manual MCP handshake: initialize → tools/list → 38 tools).

## Prevention

When MCP tools fail with `context canceled`:

1. **Retry once** — the server may recover
2. **If still down**, switch to filesystem fallback immediately:
   - Read files via `read_file`, `grep`, `glob`
   - Validate manually: check frontmatter, links, structure
   - Write files directly using `write_file` or `edit_file`
3. **Create a bug task** documenting:
   - The failing tool name and parameters
   - The exact error message
   - The workaround used
4. **Check the server binary** — try direct stdio test:
   ```bash
   echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | ./target/debug/wm-cli.exe mcp
   ```
5. **Kill stale processes** — leftover `wm-cli.exe` or `wm.exe` may block fresh spawns
6. **Check `.mcp.json` path** — wrong path or missing binary causes silent failure

## Time lost

~15 minutes of retries and diagnosis before switching to fallback. Would have been 2 minutes with immediate fallback + bug task.

## Related

- @doc/rules/tool-reliability-bug-tracking — The rule violated
- @doc/patterns/dev-workflow-target-binary-mcp — How to ensure the MCP binary is the right one
- @doc/patterns/mcp-first-files-fallback — MCP-first, files-fallback pattern
