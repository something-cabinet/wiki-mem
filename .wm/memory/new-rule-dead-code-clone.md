---
type: memory
layer: project
tags: [rule, dead-code, clone, quality, rust, linting, audit, deepwork]
summary: "Complete cleanup: removed all #[allow(dead_code)], unused vars, clippy suppressions; audited 59 Clone derives (removed 41); fixed std::fs→tokio::fs, to_string→into, early collects, unwraps; zero all #[allow(...)], zero warnings. Extracted patterns/decisions/failures."
---

## Deepwork Session: Dead Code & Clone Cleanup

**Date:** 2026-07-24

### What was done

**Phase 1 — Suppression fixes (19 files):**
- Removed `#[allow(dead_code)]` from 15 sites (MCP schema fields use `_schema` flatten pattern)
- Removed `#[allow(unused_variables/mut)]` from 5 sites
- Fixed 4 clippy lint suppressions (named lints + comments)
- Removed 2 dead functions (`memory_dir`, `resolve_root`)
- Oracle gate caught 3 P1 regressions — all fixed

**Phase 2 — Clone audit (41 types cleaned, 18 kept):**
- Removed Clone from all parser DTOs (7), version models (5), config models (8), skill TriggerEvent (1), engine DTOs (14), search models (4), EmbedError (1)
- Kept Clone on types needed for petgraph, async dispatch, Copy derives, or transitive field relationships
- `.clone()` call analysis: ~310 calls total — Arc clones are free, Vec<SectionDoc> clones are the only high-cost pattern

### Artifacts created/updated

| Artifact | Type | Location |
|----------|------|----------|
| No Dead Code & Clone Optimization | Rule | `.wm/wiki/rules/no-dead-code-clone-scanning.md` |
| Zero Tolerance for Warnings (reconciled) | Rule | `.wm/wiki/rules/no-warnings.md` |
| Arc<Vec<T>> Clone-on-Write | Pattern | `.wm/wiki/patterns/arc-vec-section-corpus.md` |
| Parser take() Over clone() | Pattern | `.wm/wiki/patterns/parser-take-over-clone.md` |
| Dead Code & Clone Cleanup | Spec | `.wm/wiki/specs/dead-code-clone-cleanup.md` |

### Key findings

- `.clone()` has 5 categories: Arc (free), String/id (necessary), parser extraction (fixable with take()), Vec<SectionDoc> (expensive — use Arc::make_mut()), config field (cheap)
- The two `Vec<SectionDoc>::clone()` in `graph/mod.rs:244,297` are the single most expensive clones
- `cargo check --all-targets` must be used, not just `cargo check` — test code can have different compilation issues
- `#[serde(flatten)]` + `_schema` prefix is the canonical pattern for MCP schema fields
- Removing Clone from struct fields has transitive effects through the type tree

### Related
- `no-warnings.md` — reconciled to match the stricter rule
- `no-dead-code-clone-scanning.md` — `.clone()` call category table
- `rust-anti-patterns.md` — unwrap/expect/literal-string/early-collect/blocking-async baselines
- `fix-clone-calls.md` — spec for graph Vec clone + parser clone fixes
- `rust-anti-patterns.md` — new rule covering unwrap, expect, "literal".to_string(), early collects, blocking async I/O, with codebase baselines (243 unwrap, 354 expect, ~75 to_string, 0 unsafe)
- `fix-clone-calls.md` — spec for Vec<SectionDoc> clone and parser clone fixes (both now implemented)
