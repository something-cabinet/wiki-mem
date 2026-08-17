---
title: Fix MCP protocolVersion negotiation — negotiate highest mutually-supported
type: task
id: wiki:tasks:fix-mcp-protocolversion-negotiation--negotiate-highest-mutually-supported
status: todo
priority: medium
tags: [bug, mcp, wm-server, linus-remediation]
acceptance_criteria:
  - text: "initialize_result negotiates the highest mutually-supported protocol version (2025-06-18) instead of hardcoding a 2024-11-05 fallback"
  - text: "Conformance test (scripts/mcp-conformance) still passes end-to-end"
  - text: "mcp_http + mcp_test suites green; clippy clean"
---

T5 finding (reported, not fixed): SDK 1.30 sends protocolVersion 2025-11-25 (its LATEST_PROTOCOL_VERSION); apps/wm-server/src/routes/mcp.rs initialize_result does not recognize it and hardcodes a fallback to 2024-11-05 instead of negotiating to the highest mutually-supported version (2025-06-18). Client tolerates the downgrade today, but a future SDK dropping 2024-11-05 would break connect(). Fix the negotiation; re-run the conformance script.