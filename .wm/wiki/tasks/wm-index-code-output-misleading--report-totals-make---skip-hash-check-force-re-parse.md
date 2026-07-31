---
title: wm index code output misleading — report totals, make --skip-hash-check force re-parse
type: task
tags:
- from-finding
- cli
- code-intel
status: in-progress
priority: medium
relates_to:
- type: implements
  target: wiki:specs:code-index-cli-output-totals-vs-delta
acceptance_criteria:
- text: 'wm index code output distinguishes totals vs delta: prints files scanned, files changed (re-parsed), total symbols in index (+N new), total deps (+N new) so a no-change run is clearly ''0 new'' not ''0 total'''
  checked: false
- text: wm index rebuild code-index line reports the same totals+delta semantics
  checked: false
- text: '--skip-hash-check (or a new --force flag) actually forces full re-parse: clears/invalidates code.db hashes so every file is re-indexed, verified by output showing re-parse of all files'
  checked: false
- text: No-change run on a populated project exits 0 with output that clearly states index is current (e.g. '37354 symbols in index (0 new)')
  checked: false
- text: cargo test -p wm-code-intel passes; unit test covers delta-vs-total output semantics
  checked: false
implementation_plan: |-
  1. Add `CodeIndexStats` struct (files_scanned, files_changed, symbols_indexed, deps_indexed, total_symbols, total_deps, errors) in packages/wm-code-intel/src/models/ following symbol_model.rs pattern + mod.rs barrel export.

  2. `rebuild_code_index(db, root, force: bool) -> Result<CodeIndexStats, String>`:
     - `force == false` → existing mtime/sha256 skip; `force == true` → bypass skip, re-parse all files
     - files_changed = changed_data.len(); deltas from changed_data (as today); totals via new CodeIndexDb count methods after upsert

  3. code_index_db.rs: add `count_symbols() -> Result<usize, _>` and `count_deps()` (mirror get_file_count_and_max_mtime style; COUNT(*) queries)

  4. main.rs IndexAction::Code { skip_hash_check }: wire `force: skip_hash_check`, remove inert ack message, print new format: files scanned / files changed / "N symbols in index (+M new)" / "N dependencies in index (+M new)". Same format for the wm index rebuild code-index line (main.rs:2693-2705).

  5. TDD: update existing tests (add `false` arg, destructure stats fields), add tests: force re-parses unchanged file; totals reflect DB totals after no-change run.
---

Finding: `wm index code` (v0.3.6, apps/wm-cli/src/main.rs:2714-2751 + packages/wm-code-intel/src/services/ingest_service.rs:137-141) prints DELTA counts ("N symbols indexed") from changed_data only. On a no-change incremental run it prints "0 symbols indexed / 0 dependencies indexed" even though code.db holds 37354 symbols / 20370 deps — reads as broken/empty to users. Verified live on eightcap-new-portal: index healthy (7230 files: 6394 typescript, 800 html, 36 python); touching app.module.ts → run reports "1 symbols indexed, 14 dependencies indexed" proving TS pipeline works. Also: IndexAction::Code { skip_hash_check } flag (main.rs:2715-2717) is acknowledged but inert — users expecting a forced full rebuild get a silent no-op.