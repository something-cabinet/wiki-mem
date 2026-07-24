---
id: wiki:specs:domain-splits-page-codeintel-template-graph
title: Domain Splits: page.rs, code_intel.rs, template_engine.rs, graph.rs
type: spec
tags: [spec, refactor, domain, module-structure]
status: draft
---
id: wiki:specs:domain-splits-page-codeintel-template-graph

## Overview

Split 4 monolithic files into domain-oriented subdirectories. No logic changes — pure file reorganization with `mod.rs` re-exports.

## Rule

Each top-level concern in a file should be its own sub-module. "What" comments that act as section markers (`// ─── Name ───`) are the primary signal — where a file has 4+ section markers covering distinct concerns, those sections become files.

## Splits

### 1. code_intel.rs → code_intel/

**Why:** 944 lines, 7 language parsers, engine, debug tools all mixed.

**Structure:**
```
code_intel/
  mod.rs     — re-exports, public API (extract_symbols, extract_deps, infer_language)
  types.rs   — CodeIntelSymbol, CodeIntelDep
  language.rs — SupportedLanguage enum, from_ext, load_language
  engine.rs  — CodeIntelEngine, LSP config, grammar init
  parser.rs  — get_or_create_parser, parse_source, compile_query, run_query
  tests.rs   — all tests
```

**Risk:** Low — feature-gated behind `#[cfg(feature = "code-intel")]`.

### 2. template_engine.rs → template_engine/

**Why:** 535 lines, render loop + block parsing + variable resolution + helpers mixed.

**Structure:**
```
template_engine/
  mod.rs       — re-exports, RenderResult, render_template
  block.rs     — extract_block, tag parsing
  variable.rs  — resolve_variable, resolve_condition, is_truthy
  helpers.rs   — apply_helper, to_pascal_case, to_camel_case etc.
  tests.rs     — all tests
```

**Risk:** Low-Med — well-isolated, only `render_template` and `RenderResult` are pub.

### 3. graph.rs → graph/

**Why:** 370 lines, graph build + section build + index gen + lint + path finding mixed.

**Structure:**
```
graph/
  mod.rs       — re-exports, rebuild_graph_snapshot
  build.rs     — build_graph_from_wiki, edge validation
  sections.rs  — build_sections_from_wiki
  index_gen.rs — auto_generate_index
  lint.rs      — auto_fix_missing_frontmatter
  path.rs      — find_path
```

**Risk:** Medium — widest consumer base (10+ callers), must preserve exact API.

### 4. page.rs → page/

**Why:** 796 lines, page CRUD + YAML mutation + path resolution + JSON migration + orphan recovery mixed.

**Structure:**
```
page/
  mod.rs      — re-exports, module documentation
  crud.rs     — create_page, get_page, get_page_raw, list_pages, delete_page
  update.rs   — update_page, PageUpdateParams
  yaml.rs     — parse_yaml_mut, set_yaml_field, extract_yaml_string_value, ac_set_checked, remove_yaml_block
  path.rs     — resolve_page_path, resolve_id_to_path, resolve_simple_page_path
  migration.rs — migrate_old_memory_json
  recovery.rs — recover_orphan_timers
  tests.rs    — all tests
```

**Risk:** Medium — 11 callers across 4 files, central to app.

## Acceptance Criteria

- [ ] AC-1: code_intel.rs split → all feature-gated builds pass
- [ ] AC-2: template_engine.rs split → template tests pass
- [ ] AC-3: graph.rs split → all 10 callers work unchanged
- [ ] AC-4: page.rs split → all 11 callers work unchanged
- [ ] AC-5: All 227 tests pass after all splits
- [ ] AC-6: cargo clippy reports no new warnings

## Execution order

1. `code_intel.rs` (lowest risk, warm-up)
2. `template_engine.rs` (well-isolated)
3. `graph.rs` (medium risk, many callers)
4. `page.rs` (highest complexity, save for last)