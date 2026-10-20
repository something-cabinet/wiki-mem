---
title: Short lock windows with transactional derived-index writes
type: memory
tags: [concurrency, transaction, index, architecture, code-intel]
status: active
relates_to:
  - {type: references, target: wiki:patterns:short-lock-window-transactional-index-writes}
---

When a local DB takes a process-wide lock at open, hold the handle only for cheap reads and the write apply: hash -> drop -> scan/extract -> reopen. Use one BEGIN IMMEDIATE transaction per logical write; delete derived rows inside the base transaction so a crash yields (old base + old derived) or (new base + empty derived), never (new base + old derived). Write an intent marker before mutation, single-flight the rebuild, and let readers recompute when derived rows are empty. Full reference: @wiki/patterns/short-lock-window-transactional-index-writes