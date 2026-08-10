---
title: MCP input_schema top-level oneOf — violates spec, client rejects tools/list
type: task
tags:
- bug
- mcp
- schemas
- tool-reliability
status: done
priority: high
acceptance_criteria:
- text: No wm tool input_schema has oneOf/allOf/anyOf at the top level (checked across all registered tools)
  checked: false
- text: Tagged action-enum tools (page, task, template, memory, model, decision, time, source, doc) emit MCP-compliant object schemas (action as a required property + variant fields)
  checked: false
- text: tools/list validates against MCP spec (client no longer rejects connection)
  checked: false
- text: test_regression_wm_page_schema_complete updated (no longer asserts top-level oneOf)
  checked: false
- text: New regression test asserts no top-level composition keywords in any registered tool schema
  checked: false
- text: cargo check --workspace + clippy clean; cargo test -p wm-core green
  checked: false
relates_to:
- type: implements
  target: wiki:specs:mcp-input-schema-no-top-level-composition
---

MCP client rejects tools/list: "tools.18.custom.input_schema: input_schema does not support oneOf, allOf, or anyOf at the top level". Root cause: 9 tools (page, task, template, memory, model, decision, time, source, doc) register via register_typed with tagged action enums (#[serde(tag = "action")]); generate_input_schema (apps/wm-core/src/mcp/transport.rs:39) uses schemars into_root_schema_for which emits a top-level oneOf for tagged enums. MCP spec forbids composition keywords at the input_schema root. The existing test_regression_wm_page_schema_complete (mcp_test.rs) asserts the invalid top-level oneOf shape and must be corrected.