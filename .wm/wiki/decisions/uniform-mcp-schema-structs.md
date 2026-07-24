---
id: wiki:decisions:uniform-mcp-schema-structs
title: "Decision: Uniform Schema Structs for MCP Tool Actions"
type: decision
status: approved
tags: [decision, mcp, schema, uniformity]
decision:
  context: "Action enums mix two patterns: inline variant fields (for handler-used params) and #[allow(dead_code)] (for schema-only fields). Every new variant requires a judgment call."
  options:
    - "Current mixed pattern (inline + #[allow])"
    - "Uniform schema structs for every variant"
  rationale: "Uniformity eliminates judgment calls. Every variant with parameters gets a schema struct. New devs never wonder which pattern to use. Zero #[allow(dead_code)]."
  outcome: "Every MCP tool action variant with parameters gets a dedicated schema struct named Wm{Domain}{Variant}Schema."
---
id: wiki:decisions:uniform-mcp-schema-structs
