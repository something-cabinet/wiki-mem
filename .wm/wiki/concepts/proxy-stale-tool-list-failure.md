---
{}
relates_to:
  - {type: references, target: wiki:tasks:mcp-direct-t1-replace-proxy}
  - {type: references, target: wiki:specs:mcp-direct-handlers}
  - {type: references, target: wiki:decisions:mcp-direct-handlers-over-proxy}
  - {type: references, target: wiki:learnings:proxy-architecture-single-entrypoint}
  - {type: references, target: wiki:patterns:mcp-proxy-singleton}
  - {type: references, target: wiki:tasks:srv-create-mcp-proxy-with-static-tool-list}
  - {type: references, target: wiki:concepts:hlmselect-portal-ng-container}
  - {type: references, target: wiki:concepts:mcp-tool-unavailability-fallback}
  - {type: references, target: wiki:concepts:schema-error-tagged-enums}
  - {type: references, target: wiki:concepts:wm_page-tags-bug}
  - {type: references, target: wiki:concepts:missed-project-guidance-fjadra}
---

---
title: Failure: Proxy STATIC_TOOLS silently rotted
type: concept
tags: [failure, mcp, proxy, maintenance]
---

## What went wrong

The `mcp_proxy.rs` file maintained a `STATIC_TOOLS` list of 50 tool names used to register proxy handlers. Over ~2 months of development, the tool architecture was consolidated (action-enum pattern: `wm_page` instead of `wm_page.get`/`wm_page.create`), but `STATIC_TOOLS` was never updated. An oracle review found:
- ~26 of 50 advertised names no longer existed in the engine registry
- ~25 registered tools were unreachable via the proxy
- All 50 tools had empty descriptions and empty input schemas

## Root cause

The `STATIC_TOOLS` list was a separate source of truth from `register_all_tools()`. There was no automated check that the two matched. Tool renames and consolidations in `wm-core` never propagated to the proxy list in `wm-cli`.

## Prevention

- Don't maintain duplicate tool registration sources. There must be exactly one: `register_all_tools()`.
- If a proxy layer is unavoidable, generate the tool list dynamically from the registry, not from a static array.
- Add a test that verifies `tools/list` output matches expectations across refactors.

## Time lost

~30 min debugging + 1.5h of oracle review to quantify the drift. Would have caused hours of confusion per incident for any developer adding a tool and wondering why it didn't appear in MCP.

## Related

- @wiki/tasks/mcp-direct-t1-replace-proxy
- @wiki/specs/mcp-direct-handlers