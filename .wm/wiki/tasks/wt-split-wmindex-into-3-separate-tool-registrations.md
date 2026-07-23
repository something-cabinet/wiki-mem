---
title: WT: Split wm_index into 3 separate tool registrations
type: task
status: todo
priority: high
tags: [spec:wiki-tool-reliability, mcp, refactor]
---

Rewrite apps/wm-core/src/mcp/tools/index.rs: replace single WmIndexAction tagged enum with 3 separate register_typed calls: wm_index_rebuild, wm_index_status, wm_index_embed. Each has its own input struct. This eliminates the hidden action discriminator and the force: _ discard.