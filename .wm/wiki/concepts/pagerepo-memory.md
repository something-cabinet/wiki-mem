---
id: wiki:concepts:pagerepo-memory
title: PageRepo — Repository Trait for Filesystem I/O
type: concept
tags: [pattern, testing, filesystem]
relates_to:
  - {type: references, target: wiki:patterns:pagerepo-trait}
---
id: wiki:concepts:pagerepo-memory

Extract filesystem I/O behind a `PageRepo` trait with two impls: `FsPageRepo` (prod) and `InMemoryPageRepo` (tests). Public API stays backward-compatible via internal delegation. Pattern applied to `page.rs` — 7 functions refactored.

Full reference: @wiki/patterns/pagerepo-trait