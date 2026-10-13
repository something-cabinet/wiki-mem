---
title: wm_page.link races under concurrency and clobbers relates_to
type: memory
tags: [tool-reliability, graph, frontmatter, mcp]
status: active
---

wm_page.link has two defects (reproduced 2026-10-13, task wiki:tasks:wmpagelink-concurrency-race-and-relates-to-clobber). (1) Concurrent link calls on the SAME source page race on the frontmatter rewrite: 4 of 5 parallel calls failed with IO_ERROR 'No such file or directory (os error 2)'; the same calls sequentially all succeeded. (2) link REPLACES the page's relates_to block instead of appending, so it silently drops existing edges, including ones authored directly in frontmatter — 1 existing edge + 4 sequential link calls left exactly 1 edge. Workaround: author relates_to directly in frontmatter and never call link concurrently; if used, verify the file after each call. Same silent-success class as the check_ac write bug (rule: @wiki/rules/durable-write-or-error).