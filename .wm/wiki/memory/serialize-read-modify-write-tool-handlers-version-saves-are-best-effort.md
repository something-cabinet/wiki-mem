---
title: Serialize read-modify-write tool handlers; version saves are best-effort
type: memory
tags: [concurrency, durability, tool-reliability, frontmatter]
status: active
relates_to:
  - {type: references, target: wiki:specs:record-format-wave}
---

For a tool handler that reads a file, mutates it, and writes it back: hold a poison-recovering process-wide (or per-page) mutex across the whole read-modify-write, perform the durable page write first, then treat version/audit side-writes as best-effort — log a warning on failure, never return IO_ERROR after the page landed. A pre-write version save with `?` can abort a valid update and concurrent RMW calls can lose notes. Reference: @wiki/specs/record-format-wave; regression page_update_concurrent_writes_do_not_return_io_error.