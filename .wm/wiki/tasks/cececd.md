---
title: Refactor wm-server to accept ToolRegistry externally
id: cececd
type: task
status: cancelled
priority: high
tags: [from-spec, mcp, server]
spec: specs/mcp-tool-registry-unification
relates_to:
  - {type: implements, target: wiki:specs:mcp-tool-registry-unification}
---

Remove the OnceLock<Arc<ToolRegistry>> global from wm-server. Add registry to AppState. Change build_api_router_with to accept Arc<ToolRegistry> as parameter. Provide convenience build_api_router that creates its own registry.
