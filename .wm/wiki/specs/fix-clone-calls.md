---
id: wiki:specs:fix-clone-calls
title: Fix High-Cost .clone() Calls
type: spec
status: approved
tags: [spec, clone, performance, rust]
references: "@wiki/patterns/arc-vec-section-corpus, @wiki/patterns/parser-take-over-clone, @wiki/rules/no-dead-code-clone-scanning"
---
id: wiki:specs:fix-clone-calls

## Overview

Fix the two highest-cost `.clone()` patterns identified in the clone audit: the full `Vec<SectionDoc>` clones in graph rebuild, and the ~43 parser field clones that should use `take()`.

## Requirements

### FR-1: Fix Vec<SectionDoc>::clone() in graph/mod.rs
- FR-1.1: Replace `(*existing).clone()` at line 244 with `Arc::make_mut()` pattern
- FR-1.2: Replace `(*existing).clone()` at line 297 with `Arc::make_mut()` pattern
- FR-1.3: Preserve all existing behavior — graph traversal and search must produce identical results

### FR-2: Fix parser field clones in parser/mod.rs
- FR-2.1: Replace `.clone()` field extractions with `take()` or `mem::take()` where the frontmatter is consumed after extraction
- FR-2.2: Only change fields where the source is definitively consumed afterward
- FR-2.3: Preserve all parsing output — no behavioral changes

## Acceptance Criteria

- [x] AC-1: `cargo check --workspace --all-targets` passes
- [x] AC-2: `cargo test --workspace --lib` passes
- [x] AC-3: No behavioral changes in graph or search output
- [x] AC-4: Clone count reduced (rg '\.clone\(\)' | wc -l) — ~45 fewer in parser, 2 fewer Vec clones in graph

## Technical Notes

- `graph/mod.rs:244,297`: The corpus is stored as `Arc<Vec<SectionDoc>>`. `Arc::make_mut()` clones-on-write — zero-copy when sole owner.
- `parser/mod.rs`: The frontmatter is parsed and then immediately consumed to build meta. Each `.clone()` on a `String`/`Vec` field can be replaced with `std::mem::take()` or direct move.
