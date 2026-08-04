---
title: Critical Patterns
type: core
tags: [critical]
status: active
---

---
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


## 2026-07-27 RuleCategory Enum Missing Variants Silently Drops Entire Frontmatter

**Category:** failure
**Source:** @wiki/decisions:silent-err-catch-in-parsers
**Tags:** [parser, serde, enum, silent-failure, frontmatter]

The `RuleCategory` enum in `packages/wm-engine/src/models/page_data/rule_category_model.rs` only had 9 variants (Naming, Branching, Design, etc.). When a rule file used `category: workflow` or `category: quality` in its YAML frontmatter, `serde_yaml` failed to deserialize the entire `Frontmatter` struct. The error was silently swallowed by `Err(_)` in `extract_frontmatter()`, returning `None` for the frontmatter. This caused `parse_wiki_page` to default to `PageType::Concept`, making 4 of 8 rule files invisible to `wm_page.list({"type": "rule"})`.

**Fix:** Add missing enum variants (`Workflow`, `Quality`) to `RuleCategory`. Also changed `Err(_)` to `Err(e)` with `tracing::warn!` in `extract_frontmatter()` so future parsing errors are visible in logs.

**Full entry:** @wiki/decisions/silent-err-catch-in-parsers


## 2026-07-27 indicatif Spinner Requires enable_steady_tick to Animate

**Category:** failure
**Source:** @wiki/concepts:spinner-without-steady-tick
**Tags:** [cli, indicatif, progress, animation]

`ProgressBar::new_spinner()` creates a stopped spinner. Without `enable_steady_tick(duration)`, the spinner draws exactly one static frame and never animates — functionally identical to a bare `println!`. The blocking/sync work function prevents any tick advancement.

**Fix:** Always call `enable_steady_tick(std::time::Duration::from_millis(100))` immediately after `new_spinner()`. This spawns a background thread that drives animation ticks regardless of main-thread blocking.

```rust
let spinner = ProgressBar::new_spinner();
spinner.set_style(ProgressStyle::default_spinner()...);
spinner.enable_steady_tick(Duration::from_millis(100)); // required
spinner.set_message("Working...");
```

**Full entry:** @wiki/concepts:spinner-without-steady-tick


## 2026-07-31 cargo-npm bins Only Accepts Same-Crate Binaries

**Category:** failure
**Source:** @wiki/patterns/cargo-npm-github-actions
**Tags:** [npm, ci, cargo-npm, deployment, packaging]

`cargo-npm`'s `bins` field ONLY accepts binary targets from the SAME crate. Listing a binary from another workspace crate (`bins = ["my-cli", "my-server"]`) fails CI at `cargo npm generate` with `error: unknown bin(s) ["my-server"] for '@scope/my-cli'; available: ["my-cli"]`. This caused a failed release tag (v0.3.2) and forced re-architecture from a single bundled package to one npm package per binary.

**Fix:** One `[package.metadata.npm]` section per crate. Reference secondary packages as `optionalDependencies` of the main package so `npm install -g @scope/main` pulls everything. Resolve sibling binaries at runtime by walking up from `current_exe()` scanning `node_modules/@scope/my-server-*/`.

**Full entry:** @wiki/patterns/cargo-npm-github-actions


## 2026-07-31 cargo-npm Scoped Output Dir — Non-Matching Glob Silently Skips

**Category:** failure
**Source:** @wiki/concepts/cargo-npm-scoped-output-silent-noop-glob
**Tags:** [ci, cargo-npm, glob, packaging]

`cargo-npm generate` writes platform packages to a SCOPED subdirectory: `npm/@scope/my-server-darwin-arm64/`, not the flat `npm/my-server-darwin-arm64/` shown in the docs. A bash for-loop over a non-matching glob (`for dir in npm/my-server-*/`) silently skips the body with no error — the CI step reports success while copying nothing. v0.3.5 shipped binary-only packages missing the bundled frontend.

**Fix:** Glob the scoped path `npm/@scope/my-server-*/` AND guard the loop: `[ -d "$dir" ] || { echo "no platform packages"; exit 1; }`. After the first release with a new bundle step, download the published tarball and verify the bundled assets exist — the missing-UI symptom only appears at runtime for users.

**Full entry:** @wiki/concepts/cargo-npm-scoped-output-silent-noop-glob


## 2026-07-31 Incremental Rebuild Deltas Misread as Totals — Report Totals + Delta

**Category:** failure
**Source:** @wiki/tasks:wm-index-code-output-misleading--report-totals-make---skip-hash-check-force-re-parse
**Tags:** [cli, incremental, hash-skip, ux]

Incremental/hash-skip rebuilds report only NEW items this run. `wm index code` printed "7230 files scanned, 0 symbols indexed" while code.db held 37354 symbols — a no-change run reads as a broken index and triggers a full investigation. Some files also legitimately extract 0 symbols (TS `module.exports = {...}` is an assignment expression, not captured by the symbol queries). Related: `--skip-hash-check` existed but was acknowledged without being wired.

**Fix:** CLI output must show post-run totals with deltas: "N symbols in index (+M new)". When a user reports "0 indexed", query the persisted DB directly (`SELECT COUNT(*) FROM code_symbols`) before suspecting the pipeline. Wire every CLI flag into behavior or remove it.

**Full entry:** @wiki/patterns/cli-delta-vs-total-reporting


## 2026-07-31 wm_task Store Stale for Newly Created Pages — wm_page.update Is the Authoritative Write

**Category:** failure
**Source:** @wiki/concepts/wm-task-store-stale-for-new-pages
**Tags:** [tool-reliability, mcp, task-store, staleness]

Newly created task pages return phantom `NOT_FOUND` from `wm_task.update/get/time`, and the status transition validator rejects `todo → done` even right after a successful `in-progress` update — the task store resolves IDs and validates transitions against a stale snapshot that excludes recently created tasks. `wm_page.update` with the same `wiki:tasks:...` ID works reliably (page store), and linking the task to its spec appears to unblock task-store ID resolution.

**Fix:** When `wm_task.*` misbehaves on a freshly created task, write status via `wm_page.update`, link the task → spec, and validate via `wm_validate.check` on the entity. Part of the known tool-reliability bug set (task @wiki/tasks/7ce26d).

**Full entry:** @wiki/concepts/wm-task-store-stale-for-new-pages


## 2026-08-04 Path::starts_with Does Not Resolve `..` — Every Lexical Guard Is Bypassable

**Category:** failure
**Source:** @wiki/patterns/lexical-path-confinement
**Tags:** [security, path-traversal, filesystem, critical]

`Path::starts_with` is component-wise and does NOT normalize `..`. This means `.wm/wiki/../../etc/passwd.md.starts_with(".wm/wiki")` returns `true`. Every path-confinement guard that relied on lexical `starts_with` was bypassable — 6 guards across `page_path_helper.rs`, `doc.rs`, and the template runner. Four were exploited end-to-end (arbitrary read, arbitrary write, arbitrary delete, cross-origin exfiltration).

`canonicalize()` alone doesn't fix it either — create-paths don't exist on disk yet, so canonicalize returns `Err`. The fix is lexical normalization (collapse `..` without touching disk) THEN `starts_with` on the normalized result.

**Fix:** One `confine(root, candidate)` helper that normalizes lexically, checks `starts_with`, and then — if the path exists — canonicalizes and re-checks for symlink escape. `confine_strict` additionally rejects dot-components (for `.git/config` exposure when secrets sit inside the root). Applied at all 9 previously-broken sites. See `apps/wm-core/src/shared/helpers/path_confine_helper.rs`.

**Full entry:** @wiki/patterns/lexical-path-confinement


## 2026-08-04 .gitignore Does Not Support Inline Comments

**Category:** failure
**Source:** @wiki/concepts/failure-bulk-frontmatter-repair-data-loss
**Tags:** [git, gitignore, silent-failure]

`**/.wm/skills/    # synced from embedded binary` is a pattern matching files ending with `    # synced from embedded binary`, not `**/.wm/skills/`. Three patterns in this repo were dead for months — `.wm/skills/`, `.wm/log.jsonl`, and `.wm/state/web-token` were never actually ignored. Only `.wm/audit.jsonl` worked because it had no trailing comment.

**Fix:** Comments must be on their own line. After fixing, verify with `git check-ignore -q <path>`. Also: a corrected pattern doesn't apply to already-tracked files — `git rm --cached` is needed separately.

**Full entry:** See `.gitignore` fix in commit ee4bdff
