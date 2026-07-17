---
title: Fix 9 MCP tool schemas missing root type: object
type: task
status: todo
---

**Severity:** Medium

**Observed:** 7 MCP tools have schemas missing root `"type": "object"`, falling back to empty schemas: wm_page, wm_source, wm_index, wm_task, wm_model, wm_time, wm_decision, wm_memory, wm_template.

**Root Cause:** The schemars derive or manual schema definitions for these tools don't include the required root `"type": "object"` field. MCP spec requires inputSchema to have root type 'object'.

**Acceptance Criteria:**
- [ ] All MCP tools have valid input schemas with root `"type": "object"`
- [ ] No "Failed to generate schema" errors on startup