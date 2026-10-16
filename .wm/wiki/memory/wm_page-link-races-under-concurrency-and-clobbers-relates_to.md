---
title: wm_page.link races under concurrency and clobbers relates_to
type: memory
tags: [tool-reliability, graph, frontmatter, mcp]
status: active
---

schema_version: 1
state: |-

  wm_page.link had two reproduced defects (2026-10-13, task wiki:tasks:wmpagelink-concurrency-race-and-relates-to-clobber).

  (1) Concurrent link calls on the SAME source page raced on the frontmatter rewrite: 4 of 5 parallel calls failed with IO_ERROR 'No such file or directory (os error 2)' (temp path was ".{file}.tmp-{pid}", not unique per call). Sequential calls all succeeded.
  (2) link REPLACED the page's relates_to block instead of appending, silently dropping existing edges including frontmatter-authored ones (1 existing edge + 4 sequential link calls left exactly 1 edge).

  FIXED 2026-10-13 (commit e437059, spec wiki:specs:wmpagelink-merge-and-write-serialization): FsPageRepo::write now uses a per-call unique temp name (pid + AtomicU64); link_page does the read-merge-write inside a process-wide per-path lock; wm_page.link appends and dedupes by (type, target) and verifies the edge on disk; wm_page.update keeps replace semantics. Pattern: @wiki/patterns/atomic-file-writes-unique-temp-and-path-lock Decision: @wiki/decisions/link-merge-vs-update-replace

NOTE: until the binary is rebuilt, the running 0.5.2 server still clobbers relates_to on link — author relates_to directly in frontmatter and rebuild the index instead.