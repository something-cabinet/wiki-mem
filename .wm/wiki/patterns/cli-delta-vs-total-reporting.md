---
title: 'Pattern: CLI Delta-vs-Total Reporting'
id: wiki:patterns:cli-delta-vs-total-reporting
type: pattern
relates_to:
  - {type: relates_to, target: wiki:patterns:hash-skip-rebuild}
  - {type: references, target: wiki:tasks:wm-index-code-output-misleading--report-totals-make---skip-hash-check-force-re-parse}
status: reviewed
tags: [pattern, cli, ux, incremental]
---

# Pattern: CLI Delta-vs-Total Reporting

## Problem

CLI commands that operate incrementally (index rebuilds, syncs, scans) usually print how much NEW work this run did (deltas). Users read "N symbols indexed" as the size of the index. On a no-change run the delta is 0, so the output reads as a broken/empty tool even when the underlying store holds tens of thousands of entries. Reproduced live: `wm index code` printed "7230 files scanned, 0 symbols indexed" while code.db actually held 37354 symbols / 20370 deps.

## Solution

Return a stats struct carrying BOTH the delta and the post-run totals, and print totals with the delta in parentheses:

```
N files scanned
N files changed
N symbols in index (+M new)
N dependencies in index (+M new)
```

- Totals must be queried from the persisted state AFTER the write (post-upsert), not accumulated from the run — they reflect the final DB, not the run's work.
- Deltas come from the changed set; totals come from the store.
- Provide a `force` escape hatch that bypasses the incremental skip so users can trigger a full re-parse when in doubt (wire the flag to it — see @wiki/concepts/inert-cli-flags-silent-noop).

```rust
// ingest_service.rs
pub struct CodeIndexStats {
    pub files_scanned: usize,
    pub files_changed: usize,   // re-parsed this run
    pub symbols_indexed: usize, // delta
    pub deps_indexed: usize,    // delta
    pub total_symbols: usize,   // post-run DB count
    pub total_deps: usize,      // post-run DB count
    pub errors: Vec<String>,
}
```

## When to Use

- Any CLI reporting results of an incremental/hash-skip operation
- Output is consumed by humans or agents who don't know the internals
- A "no-change" run is a normal, frequent path

## When Not to Use

- One-shot commands that always do full work (delta == totals every run)
- Pure streaming/progress output where per-item events are the interface

## Related

- @wiki/patterns/hash-skip-rebuild
- @wiki/specs/code-index-cli-output-totals-vs-delta
- @wiki/concepts/incremental-rebuild-zero-delta-false-alarm