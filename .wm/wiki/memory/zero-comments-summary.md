---
title: "Zero Comments — Extract Over Document"
type: memory
tags: [naming, quality, rule]
created_at: "2026-07-24"
relates_to:
  - {type: references, target: wiki:decisions:zero-comments-extract-over-document}
---

No `//`, `///`, `//!`, `/** */` in source. If it needs a comment, split/rename instead. TODOs → WM tasks. Enforce via `rg '^\s*//'`. Full reference: @wiki/decisions/zero-comments-extract-over-document
