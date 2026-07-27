---
{}
relates_to:
  - {type: references, target: wiki:patterns:rust-binary-integration-test}
  - {type: relates_to, target: wiki:decisions:mcp-prefix}
---

id: wiki:patterns:mcp-response-format

## When to use

When building an MCP server (Model Context Protocol) in any language. The MCP spec defines precise JSON-RPC 2.0 response shapes for each endpoint. Getting these wrong causes silent failures in MCP clients (OpenCode, Claude Code, Codex).

## Response formats

### initialize response

The `result` field must contain protocolVersion, serverInfo, capabilities, and optional instructions:

```json
{
  "result": {
    "protocolVersion": "2024-11-05",
    "serverInfo": { "name": "my-server", "version": "1.0" },
    "capabilities": { "tools": {} },
    "instructions": "Call wm_initial first."
  }
}
```

### tools/list response

The `result` field must wrap tools in a `tools` key:

```json
{
  "result": {
    "tools": [
      { "name": "tool_name", "description": "...", "inputSchema": { "type": "object", "properties": {} } }
    ]
  }
}
```

Directly returning an array in `result` will fail — MCP clients expect `result.tools`.

### tools/call response

The `result` field must contain a `content` array of content items:

```json
{
  "result": {
    "content": [{ "type": "text", "text": "{\"key\": \"value\"}" }]
  }
}
```

### Error response

JSON-RPC errors use the standard format with `code` and `message`:

```json
{
  "error": { "code": -32602, "message": "Required field missing" }
}
```

Do NOT double-wrap errors. The `ToolError::to_json()` should return `{"code": ..., "message": ...}` — the transport layer adds the outer `"error"` wrapper.

## Source

@wiki/tasks/295eir @wiki/tasks/s2ff4x