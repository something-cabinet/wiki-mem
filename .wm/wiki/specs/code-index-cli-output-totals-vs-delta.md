---
title: Code Index CLI Output — Totals vs Delta
type: spec
id: wiki:specs:code-index-cli-output-totals-vs-delta
status: draft
tags: [code-intel, cli, ux]
---

# Code Index CLI Output — Totals vs Delta

## Overview

`wm index code` / `wm index rebuild` report symbol/dep counts from the incremental rebuild's **changed** files only, so a no-change run prints `0 symbols indexed` even when the index is fully populated. Users read this as a broken/empty index. Make output show totals + delta, and give users a real way to force a full re-parse.

## Background (finding)

- Reproduced on eightcap-new-portal with published `@something-cabinet/wm-cli@0.3.6`: `wm index code` → `7230 files scanned, 0 symbols indexed, 0 dependencies indexed`, exit 0.
- `code.db` was actually healthy: 7230 files (6394 typescript, 800 html, 36 python), 37354 symbols, 20370 deps.
- Live experiment: appending newline to `app.module.ts` (has `export class AppModule`) → re-run reports `1 symbols indexed, 14 dependencies indexed` → TS incremental re-indexing works.
- Root cause of misleading output: `rebuild_code_index` returns `changed_data`-only sums (`packages/wm-code-intel/src/services/ingest_service.rs:137-141`); CLI prints those as if totals (`apps/wm-cli/src/main.rs:2733-2736`, `:2701-2704`).
- Side trap: `IndexAction::Code { skip_hash_check }` (`main.rs:2714-2717`) is acknowledged but inert — "hash-check behavior is always active".
- Note: files like `jest.config.ts` (`module.exports = {...}`) legitimately extract 0 symbols — TS queries capture only `function/method/class/interface/type/enum/abstract_method_signature/variable_declarator`.

## Requirements

- FR-1: `wm index code` prints files scanned, files re-parsed (changed), total symbols in index with `(+N new)`, total deps with `(+N new)`.
- FR-2: `wm index rebuild`'s code-index line uses the same totals+delta semantics.
- FR-3: `--skip-hash-check` (or `--force`) actually forces full re-parse of all files.
- FR-4: No-change run output clearly states the index is current (not "0 indexed").

## Acceptance Criteria

- [ ] AC-1: No-change run on populated index prints totals with `(0 new)` — never ambiguous "0 symbols indexed".
- [ ] AC-2: Force flag re-parses all files; output shows all files changed.
- [ ] AC-3: `cargo test -p wm-code-intel` passes with a unit test covering delta-vs-total semantics.

## References

- @wiki/tasks/wm-index-code-output-misleading--report-totals-make---skip-hash-check-force-re-parse
- @wiki/patterns/hash-skip-rebuild
- @wiki/specs/code-index-cache
