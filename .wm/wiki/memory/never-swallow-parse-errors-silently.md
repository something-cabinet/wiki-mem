---
title: Never swallow parse errors silently
type: memory
tags: [decision, parser, error-handling]
status: active
---

Parser `Err(_)` silently drops the error, making debugging require manual cross-referencing. Always log with `tracing::warn!` and include a content snippet. Fixed in `extract_frontmatter`. Decision: @wiki/decisions/silent-err-catch-in-parsers