---
title: Carve out instruction docs before a body-format migration
type: memory
tags: [migration, wiki, agent-workflow, search, graph]
status: active
---

Before a bulk doc-body format migration, carve out agent-instruction/recall pages (rule, core, memory, task, spec, index, steering) or you break instruction loading; and make search/graph record-aware (read the record `state` for sections/tags/refs) or BM25 ranking and body-link edges regress. Full: @doc/patterns/instruction-doc-carve-out-before-body-migration