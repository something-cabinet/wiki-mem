---
{}
---

id: wiki:decisions:codex-toml-config

## Context

The `"claude" | "codex"` match arm treated both platforms identically, generating `.mcp.json` with JSON `mcpServers`. Codex expects `.codex/config.toml` with TOML `[mcp_servers]` format.

## Chosen approach

Split the arm. Codex generates TOML, Claude generates JSON.

## Outcome

Both platforms get correct config. TOML writer is a simple format!() call — no dependency needed.

## Source

@wiki/tasks/wkm5xh