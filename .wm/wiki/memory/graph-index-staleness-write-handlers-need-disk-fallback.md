---
title: Graph index staleness: write handlers need disk fallback
type: memory
tags: [tool-reliability, graph-index, pattern]
status: active
---

When write handlers (update/delete/task ops) resolve pages ONLY via the in-memory graph index, a stale index causes phantom "page not found" for pages that exist on disk — while get still works. Fix: graph-first/disk-fallback resolver (resolve_page_meta) wired into all page/task write handlers. Full reference: @wiki/patterns/stale-index-disk-fallback