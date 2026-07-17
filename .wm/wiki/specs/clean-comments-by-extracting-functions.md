---
title: "Clean Comments by Extracting Functions"
page_type: spec
status: draft
tags: [spec, refactor, comments, code-quality]
---

## Overview

Remove all "what" comments from `apps/wm-core/src/` by extracting the commented code into named functions. The rule is: if a comment explains what code does, the code should instead be a function whose name says what it does.

## Rules

- Delete comments where the surrounding function name already describes the code
- Extract a named function where the comment identifies a distinct logical step
- Keep `///` doc comments on public APIs, `//!` module docs, inline field annotations
- Keep `// ─── Section ───` markers as visual file navigation

## Requirements

### Functional Requirements

- **FR-1**: `search/query.rs` — delete self-evident comments, extract marker sub-steps into helpers
- **FR-2**: `search/retrieve.rs` — extract tier-based content truncation into named functions (tier1_full_content, tier2_frontmatter, tier3_title_edge)
- **FR-3**: `search/index.rs` — delete self-evident comments (parallel indexing, score normalization are already named)
- **FR-4**: `graph.rs` — delete self-evident comments, extract `resolve_wikilink_target` and `detect_graph_cycles`
- **FR-5**: `reference.rs` — delete self-evident comments
- **FR-6**: `template_engine.rs` — delete comments in render loop (the recursive descent parser is self-documenting)
- **FR-7**: `mcp/tools/*.rs` — delete Category A comments (self-evident), keep section markers
- **FR-8**: `page.rs` — extract `remove_relates_to_target`, `handle_checked_ac`, `resolve_page_path_logic`, `build_migration_frontmatter`
- **FR-9**: `code_intel.rs` — extract `skip_long_text`, `clean_rust_use_declarations`, `verify_function_item_match`; see FR-10 for the larger refactor

### Non-Functional Requirements

- Each extraction must compile without changing behavior
- All existing tests must pass
- Function names must describe WHAT the code does, not HOW

## Acceptance Criteria

- [ ] AC-1: All `search/` "what" comments removed, code functions extracted where needed
- [ ] AC-2: All `graph.rs` "what" comments removed
- [ ] AC-3: All `template_engine.rs` inline comments removed
- [ ] AC-4: All `mcp/tools/*.rs` Category A comments removed
- [ ] AC-5: `page.rs` complex blocks extracted into named functions
- [ ] AC-6: `code_intel.rs` trivial extractions done (skip_long_text, etc.)
- [ ] AC-7: All tests pass after each file's cleanup
- [ ] AC-8: cargo clippy reports no new warnings
- [ ] AC-9: `code_intel.rs` refactored into `code_intel/` directory (separate spec)

## Note on code_intel.rs

`code_intel.rs` at ~944 lines is a deeper problem. It mixes 8 language-specific parsers, a shared engine, public API functions, and tests all in one file. The comment cleanup (FR-9) is a quick win, but the real fix is splitting into a `code_intel/` module with per-language files. This should be a separate spec — the comment cleanup is just the first step.

## Technical Breakdown

### Priority 1: Search files (low risk)

| File | Comments | Action |
|---|---|---|
| `search/query.rs` | 18 | Delete self-evident, extract tier functions |
| `search/retrieve.rs` | 9 | Extract tier functions |
| `search/index.rs` | 6 | Delete self-evident |

### Priority 2: Graph + Reference + Template Engine (low risk)

| File | Comments | Action |
|---|---|---|
| `graph.rs` | 10 | Delete self-evident, extract helpers |
| `reference.rs` | 3 | Delete self-evident |
| `template_engine.rs` | 14 | Delete self-evident in render loop |

### Priority 3: MCP tools (medium risk, many files)

| File | Comments | Action |
|---|---|---|
| `mcp/tools/task.rs` | 24 | Delete self-evident, keep section markers |
| `mcp/tools/template.rs` | 30 | Delete self-evident, keep section markers |
| `mcp/tools/page.rs` | 7 | Delete self-evident, keep section markers |
| `mcp/tools/memory.rs` | 8 | Delete self-evident |
| `mcp/tools/doc.rs` | 11 | Delete self-evident |
| `mcp/tools/source.rs` | 7 | Delete self-evident |
| `mcp/tools/validate.rs` | 7 | Delete self-evident |
| `mcp/tools/code.rs` | 6 | Delete self-evident |

### Priority 4: Complex files (high risk)

| File | Comments | Action |
|---|---|---|
| `page.rs` | 8 complex | Extract: remove_relates_to, handle_checked_ac, etc. |
| `code_intel.rs` | 3 trivial | Extract: skip_long_text, clean_rust_use, verify_function_item |

### Priority 5: code_intel module split (separate spec)

Split `code_intel.rs` into `code_intel/mod.rs` with per-language files (`rust.rs`, `typescript.rs`, `python.rs`, `go.rs`, `html.rs`, `svelte.rs`), shared engine, query helpers, and tests. Order of magnitude larger effort.
