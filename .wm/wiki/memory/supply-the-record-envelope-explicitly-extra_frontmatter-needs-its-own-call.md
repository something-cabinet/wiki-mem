---
title: Supply the record envelope explicitly; extra_frontmatter needs its own call
type: memory
tags: [tooling, record-format, mcp]
status: active
---

wm_page.create writes content verbatim, so plain markdown produces a prose body on record-bearing types. Pass the full envelope (schema_version/state/questions/answers). extra_frontmatter is silently discarded whenever content is supplied, on both create and update — so ACs and priority need a second call. Never write the same page twice concurrently. Full reference: @wiki/howto/author-record-pages-with-an-explicit-envelope