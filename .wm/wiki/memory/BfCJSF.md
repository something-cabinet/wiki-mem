---
title: ToolRegistry access levels are dead code
type: memory
tags: [mcp, toolregistry, dead-code]
created_at: "2026-07-14T06:39:04.982Z"
updated_at: "2026-07-14T06:39:04.982Z"
---

register_read/write/admin are identical — same impl, no permission stored, check_permission never set, dispatch() is first-match-wins. The entire typed.rs module (225 lines) is decorative. Can be deleted in the action-enum refactor.