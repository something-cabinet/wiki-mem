---
title: TD-12 MCP touch flags old-format record pages
type: task
id: "wiki:tasks:td-12-mcp-touch-flags-old-format-record-pages"
status: done
priority: high
tags: [from-spec, spec:typed-decision-doc-format, engine, ux]
spec: specs/typed-decision-doc-format
acceptance_criteria:
  - text: "Read of an old-format record-bearing page returns an actionable migration hint (non-fatal)"
  - text: "Write (create/update) of a record-bearing page with a prose/invalid body tells the caller to fix it"
  - text: "Valid records and excluded types produce no hint"
  - text: "Tests cover get + update hint for the known empty-body page and a prose body"
relates_to:
  - {type: relates_to, target: wiki:specs:typed-decision-doc-format}
---

MCP tools must tell the user to fix an old-format record-bearing page when they touch it (read or write). Old-format = frontmatter type is record-bearing but the body does not parse as a typed-decision record. Non-fatal hint with the migration command / schema link.