---
title: "Decision: Prefixed MCP Tools (wm_)"
type: decision
tags: [mcp, naming, protocol]
status: reviewed
confidence: high
relates_to:
  - {type: implements, target: "wiki:specs:local-knowledge-engine-rust"}
---

## Context

MCP tools are registered with flat names like `search.query`, `page.create`, `code.find`. Host apps (OpenCode, Kiro, Claude Code) may have built-in tools with the same generic names. When both the host and an MCP server register `code.find`, the agent can't distinguish them.

## Chosen approach

Namespace ALL MCP tools with a project-specific prefix: `wm_search.query`, `wm_page.create`, `wm_code.find`. The prefix follows OpenCode's `{server}_{tool}` convention but is explicit in the tool registration, not relying on the client to namespace.

```json
{
  "name": "wm_search.query",
  "description": "Triple-mode search: keyword (BM25), semantic (cosine), hybrid (RRF)"
}
```

## Alternatives considered

- **No prefix**: Simpler tool names but guaranteed collisions with host apps.
- **Client-side namespace**: Rely on OpenCode's `{server}_{tool}` naming. Fragile — different clients handle this differently.
- **Prefix with version**: `wm_v1_search.query`. Unnecessary — the prefix handles disambiguation.

## Outcome

**GOOD_CALL.** No tool name collisions reported. The prefix is short (3 chars + underscore) and immediately identifies the tool's origin. The `wm_` prefix is consistently applied across all 19 registered tools.

## Source
@wiki/tasks/j4tx6c
