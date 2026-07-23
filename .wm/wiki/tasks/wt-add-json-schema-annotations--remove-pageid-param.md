---
title: WT: Add JSON schema annotations + remove page_id param
type: task
status: todo
priority: high
tags: [spec:wiki-tool-reliability, mcp, schemas]
---

1. Add #[schemars(description="...")] annotations to all wm_page.* action fields for better tool discovery
2. Remove page_id parameter entirely, canonicalize to id only
3. Verify tools/list returns complete schemas