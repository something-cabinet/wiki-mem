---
title: wm index code output misleading — report totals, make --skip-hash-check force re-parse
id: wiki:tasks:wm-index-code-output-misleading--report-totals-make---skip-hash-check-force-re-parse
type: task
status: done
acceptance_criteria:
  - text: "wm index code reports totals and delta (files scanned / files changed / symbols in index (+new) / dependencies in index (+new)) at both CLI call sites"
  - text: "--skip-hash-check forces a full re-parse and re-index instead of a silent no-op"
  - text: "CodeIndexStats model added with count_symbols/count_deps queried post-upsert; a no-change incremental run no longer reads as empty"
implementation_notes: 'Implemented + verified. Changes: CodeIndexStats model (packages/wm-code-intel/src/models/code_index_stats_model.rs + barrel), rebuild_code_index(db, root, force) -> Result<CodeIndexStats> with totals queried post-upsert (ingest_service.rs), count_symbols/count_deps on CodeIndexDb (code_index_db.rs), CLI output totals+delta at both call sites + --skip-hash-check wired as force re-parse (apps/wm-cli/src/main.rs:2693-2751). Tests: 45 pass in wm-code-intel incl. new test_rebuild_force_reparses_unchanged + test_rebuild_totals_after_no_change; e2e_code_intel 8 pass; clippy/build zero warnings; fmt clean. Smoke-verified on scratch project: no-change run prints "2 files scanned / 0 files changed / 2 symbols in index (+0 new) / 0 dependencies in index (+0 new)"; --skip-hash-check forces "2 files changed / +2 new". NOTE: change landed in the user''s own commit 4aaa517 (bundled with wiki tool reliability work; message does not mention code-index fix). Published npm 0.3.6 unaffected until next release.'
priority: medium
tags: [from-finding, cli, code-intel]
---

Finding: `wm index code` (v0.3.6, apps/wm-cli/src/main.rs:2714-2751 + packages/wm-code-intel/src/services/ingest_service.rs:137-141) prints DELTA counts ("N symbols indexed") from changed_data only. On a no-change incremental run it prints "0 symbols indexed / 0 dependencies indexed" even though code.db holds 37354 symbols / 20370 deps — reads as broken/empty to users. Verified live on eightcap-new-portal: index healthy (7230 files: 6394 typescript, 800 html, 36 python); touching app.module.ts → run reports "1 symbols indexed, 14 dependencies indexed" proving TS pipeline works. Also: IndexAction::Code { skip_hash_check } flag (main.rs:2715-2717) is acknowledged but inert — users expecting a forced full rebuild get a silent no-op.

Knowledge extracted (wm-extract): pattern @wiki/patterns/cli-delta-vs-total-reporting, failures @wiki/concepts/inert-cli-flags-silent-noop + @wiki/concepts/incremental-rebuild-zero-delta-false-alarm (references edges to this task); updated @wiki/patterns/hash-skip-rebuild with totals+delta UX property; promoted entry to @wiki/core/critical-patterns (2026-07-31); 2 project memory entries added.
