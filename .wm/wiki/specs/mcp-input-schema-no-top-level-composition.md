---
title: MCP input_schema — no top-level composition keywords
type: spec
id: wiki:specs:mcp-input-schema-no-top-level-composition
status: approved
tags: [approved, spec, mcp, schemas]
---

# MCP input_schema — no top-level composition keywords

## Overview

A strict MCP client rejects the tools/list response: `tools.18.custom.input_schema: input_schema does not support oneOf, allOf, or anyOf at the top level`. The MCP spec requires `inputSchema` to be a JSON Schema object without `oneOf`/`allOf`/`anyOf` at the root.

## Root Cause

9 tools register through `register_typed` with **tagged action enums** (`#[serde(tag = "action", rename_all = "snake_case")]`): page, task, template, memory, model, decision, time, source, doc.

`generate_input_schema::<T>()` (apps/wm-core/src/mcp/transport.rs:39) calls schemars `into_root_schema_for::<T>()`, which renders a serde tagged enum as:
```json
{ "oneOf": [ { "type": "object", "properties": { "action": { "const": "board" }, ... } }, ... ] }
```
The top-level `oneOf` violates the MCP spec, so strict clients fail the whole tools/list handshake.

## Locked Decisions

- D1: All tool input schemas must have `type: object` (or be a plain object schema) at the root — no top-level `oneOf`/`allOf`/`anyOf`
- D2: Keep the runtime serde tagged-enum parsing (action discriminator) — only the emitted JSON Schema changes
- D3: Fix centrally in `generate_input_schema` (post-process the schemars root schema) so every action-enum tool is covered without per-tool overrides
- D4: `action` becomes a required string property; each variant's fields become optional properties (union of fields); enums for action values are preserved
- D5: Existing tool names and handlers unchanged — this is a schema-shape-only change

## Requirements

### FR-1: Flatten tagged-enum schemas
In `generate_input_schema`, after building the schemars root schema: if the root object contains `oneOf` (tagged enum output), transform it into:
```json
{
  "type": "object",
  "properties": {
    "action": { "type": "string", "enum": ["board", "list", "get", ...] },
    ...union of all variant fields (all optional)...
  },
  "required": ["action"]
}
```
Variant-specific field descriptions and types must be preserved.

### FR-2: Validate all tools
Add a test that lists every registered tool and asserts its input_schema root has no `oneOf`/`allOf`/`anyOf` key and `type == "object"`.

### FR-3: Fix existing test
`test_regression_wm_page_schema_complete` (mcp_test.rs) currently asserts 7 top-level oneOf arms — update to assert the flattened shape (action property, required: ["action"], no top-level oneOf).

## Acceptance Criteria

- [ ] AC-1: No wm tool input_schema has top-level oneOf/allOf/anyOf
- [ ] AC-2: Action-enum tools emit `{"type":"object", properties:{action: {enum:[...]}, ...}, required:["action"]}`
- [ ] AC-3: tools/list accepted by strict MCP clients
- [ ] AC-4: test_regression_wm_page_schema_complete updated
- [ ] AC-5: New all-tools schema validation test added
- [ ] AC-6: cargo check/clippy --workspace clean; cargo test -p wm-core green

## References

- @wiki/tasks/mcp-inputschema-top-level-oneof--violates-spec-client-rejects-toolslist
- apps/wm-core/src/mcp/transport.rs:39 — generate_input_schema
- apps/wm-core/src/mcp/tools/{page,task,template,memory,model,decision,time,source,doc}/** — tagged action enums
- apps/wm-core/tests/mcp_test.rs — test_regression_wm_page_schema_complete
