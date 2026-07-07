---
title: P1 Rust Fixes — Tokenize, Retrieval, YAML, BM25 Perf, Truncate, BFS
description: 'Fix 10 P1 Rust issues: tokenize clones, YAML fragility, BM25 perf, duplicate code, silent errors'
createdAt: '2026-07-07T08:52:54.317Z'
updatedAt: '2026-07-07T09:01:05.765Z'
tags:
  - spec
  - approved
  - rust
  - p1
---

## Overview

Fix 10 P1 (important but non-critical) Rust issues identified by rust-reviewer. These span performance, correctness, code quality, and maintainability. None are crash/hang risks, but they compound technical debt.

## Locked Decisions

- D1: tokenize allocation — push to Vec once, not clone+push
- D2: retrieve_context should accept pre-built BM25 index as parameter, not rebuild from scratch
- D3: YAML manipulation in page.rs — migrate from line-based string matching to `serde_yaml` parse-modify-serialize
- D4: BM25 score_doc — pre-compute term frequencies per document field
- D5: rebuild_snapshot — pass custom_types as parameter, not re-read config from disk
- D6: Duplicate BFS path-finding — extract shared `graph::find_path()`
- D7: Duplicate truncate — remove from search.rs, keep util.rs version
- D8: UTF-8 index panic — replace byte index with char_indices
- D9: parse_duration_to_minutes — add `tracing::warn!` on parse failure
- D10: sync_skills_to — remove disk-based version from skill.rs, keep embed-based in main.rs

## Requirements

### Functional Requirements

- FR-1: tokenize must allocate strings efficiently (one push per token)
- FR-2: retrieve_context must not rebuild BM25 index repeatedly
- FR-3: YAML field manipulation must use structured serde_yaml parsing
- FR-4: BM25 score_doc must pre-compute term frequencies per field
- FR-5: rebuild_snapshot must accept config parameters, not re-read config.json
- FR-6: Duplicate BFS path-finding must be unified into one shared function
- FR-7: Duplicate truncate_str must be removed
- FR-8: truncate must handle multi-byte characters without panicking
- FR-9: Silent parse failures must produce visible warnings
- FR-10: sync_skills_to must have only one implementation

### Non-Functional Requirements

- NFR-1: No functional regression in BM25 scoring
- NFR-2: `cargo build` and `cargo test` pass without new warnings

## Acceptance Criteria

- [ ] AC-1: tokenize in search.rs:237,241 uses single push per token — no `word.as_str().to_string()` + `.clone()`
- [ ] AC-2: retrieve_context in search.rs:364-378 accepts pre-built bm25_index parameter
- [ ] AC-3: `set_yaml_field`, `ac_set_checked`, `remove_yaml_block` in page.rs use `serde_yaml` parse-modify-serialize
- [ ] AC-4: Bm25Index::score_doc pre-computes term frequencies per document field
- [ ] AC-5: rebuild_snapshot accepts `custom_types: &[String]` instead of re-reading config.json
- [ ] AC-6: Duplicate BFS in tools.rs:984-1043 and main.rs:1533-1558 extracted to shared `graph::find_path()`
- [ ] AC-7: Duplicate truncate function in search.rs:522-528 removed; all callers use `util::truncate_str`
- [ ] AC-8: truncate in util.rs uses `char_indices` not byte index (no panic on multi-byte chars)
- [ ] AC-9: parse_duration_to_minutes logs `tracing::warn!` on parse failure
- [ ] AC-10: Only one sync_skills_to implementation exists (embed-based in main.rs)

## Scenarios

### Scenario 1: Tokenize large document
**Given** a page with many tokens
**When** tokenize processes the text
**Then** each token string is allocated once and pushed directly to the Vec

### Scenario 2: YAML field update
**Given** a page with YAML frontmatter
**When** a field value is updated
**Then** the full frontmatter is parsed with serde_yaml, the field modified, and serialized back — no string search/replace

### Scenario 3: Truncate multi-byte text
**Given** text containing 4-byte UTF-8 characters (emoji, CJK)
**When** truncate_str is called with a byte limit
**Then** the result must be at a valid char boundary, no panic

## Technical Notes

- Items 1-2: search.rs
- Item 3: page.rs:312-374
- Item 4: search.rs:139
- Item 5: graph.rs:174-182
- Item 6: tools.rs:984-1043, main.rs:1533-1558
- Item 7: search.rs:522-528 vs util.rs:7-13
- Items 8-9: search.rs:526, tools.rs:12-24
- Item 10: skill.rs vs main.rs
