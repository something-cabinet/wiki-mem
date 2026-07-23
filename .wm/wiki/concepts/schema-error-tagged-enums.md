---
title: Failure: Tagged Enums Generate Schema Without root type: object
type: concept
---

---
title: Tagged Enums Generate Schema Without root type: object
type: concept
tags: [failure, mcp, schemas]
---

## What went wrong
All MCP tools using tagged enums (WmPageAction, WmTaskAction, etc.) logged startup errors: "Schema is missing 'type' field. MCP specification requires inputSchema to have root type 'object'."

## Root cause
`schemars::schema_for!()` generates `{"oneOf": [...], "title": "WmPageAction"}` without `"type": "object"` for tagged enums. The rmcp crate's `schema_for_input` validates this and returns Err before we can fix it.

## Prevention
Replace `schema_for_input` (from rmcp) with a custom `generate_input_schema()` that:
1. Uses schemars directly: `generator.into_root_schema_for::<T>()`
2. Adds `"type": "object"` if missing
3. Strips top-level `title`/`description`
4. Caches per TypeId

## Time lost
~30 min debugging + fix

## Related
- @task:create-wm-server-crate
