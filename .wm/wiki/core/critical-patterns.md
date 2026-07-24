---
id: wiki:patterns:critical-patterns
title: Critical Patterns
type: core
tags: [critical]
status: active
---


## 2026-07-24 Rerank Phrase Checks Must Use Raw Query, Not Stemmed Tokens

**Category:** failure
**Source:** @wiki/patterns:post-rrf-rerank
**Tags:** [search, stemming, rerank, phrase-matching]

When using Snowball stemming in the tokenizer, query tokens include both original and stemmed forms (e.g., "design patterns" → ["design", "patterns", "pattern"]). If `rerank_boost` builds phrase comparisons via `query_tokens.join(" ")`, the joined string becomes "design patterns pattern" which silently disables all phrase-level boosts — exact title (+8), starts_with (+4), contains (+2).

**Fix:** Pass the raw query string separately for phrase checks. Use stemmed tokens only for per-token checks (tag overlap, title density).

```rust
// Correct — rerank_boost takes both raw query and stemmed tokens
fn rerank_boost(doc: &IndexedDoc, query_lower: &str, query_tokens: &[String]) -> f64 {
    // phrase checks use query_lower (raw, not stemmed)
    // tag checks use query_tokens (stemmed)
}
```

Also: exact match (+8) must use Snowball stemming on both sides so "patterns"↔"Pattern" gets the full boost.

**Full entry:** @wiki/reference:search-scoring-formula


## 2026-07-25 MCP Tool API Drift Silently Breaks Integration Tests

**Category:** failure
**Source:** @wiki/concepts/test-rot-mcp-api-drift
**Tags:** [testing, mcp, api-drift, test-hygiene]

Integration tests that launch `wm-cli` as a subprocess (via `MCPClient::start`) silently rot when the MCP tool surface evolves — action-enum renames, tool renames, parameter restructuring. No compiler catches these because the test calls tools by name strings, not library functions.

**Fix:** When refactoring any MCP tool (rename, restructure action enum, add/remove params), run `rg "old_tool_name" apps/wm-core/tests/` and update all test fixtures in the same PR. Run `cargo test -p wm-core` before merging.

Also: use the `test_tools_list` test as a smoke check — it verifies essential tool names exist and catches renames on first run.

**Full entry:** @wiki/concepts/test-rot-mcp-api-drift


## 2026-07-25 Two-Layer Regression Guards: Lint + Integration Tests

**Category:** decision
**Source:** @wiki/decisions/lint-plus-integration-tests-for-wiki-health
**Tags:** [testing, lint, regression, wiki-health]

For wiki health properties that must never regress (e.g., all pages must have `id:` in frontmatter), use two layers with different trigger conditions:

1. **Lint check** (`wm_lint.check`) — run on demand, catches existing issues across all pages
2. **Integration test** (`cargo test`) — run in CI/development, catches new regressions before they merge

Each layer covers what the other misses: lint catches existing pages edited manually, tests catch new pages created via tools. Neither alone is sufficient.

**Full entry:** @wiki/decisions/lint-plus-integration-tests-for-wiki-health


## 2026-07-25 PageType parse_page_type Miss Causes Silent Concept Fallback

**Category:** failure
**Source:** @wiki/tasks:add-pagetypecore-enum-variant-and-pagecore-enum-variant
**Tags:** [parser, pagetype, enum, silent-failure]

When adding a new `PageType::Core` variant, the `parse_page_type` function in `apps/wm-core/src/parser/mod.rs` was missed. This function uses a static string match (`"task" => PageType::Task`, `"spec" => PageType::Spec`, etc.) and falls through to `_ => PageType::Concept` for unknown types. The result: all 4 core pages were silently classified as `concept` in the graph, visible only through `wm_graph_stats` where `concept` count was 112 instead of the expected 108.

**Fix:** Always update `parse_page_type()` when adding a new PageType. The 8 touch points pattern (see @wiki/patterns/page-type-registration-touch-points) lists all locations, with `parse_page_type` being the most commonly missed.

**Full entry:** @wiki/patterns/page-type-registration-touch-points
