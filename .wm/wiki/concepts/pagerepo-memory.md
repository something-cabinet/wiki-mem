---
title: PageRepo — Repository Trait for Filesystem I/O
description: Extract filesystem ops behind a PageRepo trait (FsPageRepo/InMemoryPageRepo) to unit-test file-mutating logic without EngineState bootstrap. Applied in page.rs.
page_type: note
id: concepts/pagerepo-memory
tags:
  - pattern
  - testing
  - filesystem
---

Extract filesystem I/O behind a `PageRepo` trait with two impls: `FsPageRepo` (prod) and `InMemoryPageRepo` (tests). Public API stays backward-compatible via internal delegation. Pattern applied to `page.rs` — 7 functions refactored.

Full reference: @wiki/patterns/pagerepo-trait
