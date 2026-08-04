---
title: Session Handover — Cross-Entity Search + SDD Skills
type: howto
tags: []
---
id: wiki:howto:session-handover-cross-entity-search
# Session Handover — Cross-Entity Search + SDD Skills

## Completed

- **SDD Skills**: Replaced gh-* skills with 13 wm-* workflow skills (init through template). Uses rust-embed for binary embedding. Subdirectory format (wm-*/SKILL.md). name: field parsing.
- **Platform parity**: Added antigravity, gemini, agents platform support. wm agents --sync command. 5 platform regression tests. Spec drift fixed.
- **Cross-entity search**: Per-type BM25 indexes (pages + memory). wm_search.query with type param (all/page/memory/task). RRF merge. FSRS-6 recency boost for tasks. Salience boost for critical memory. Flat text context for memory in retrieve.
- **ScoringConfig**: 12 tunable parameters in config.json search.scoring. recency_model (fsrs/linear/exponential/none). recency_stability_days.

## Key Decisions

1. WM is a memory layer, not a spec system (OpenSpec handles spec lifecycle)
2. Tasks stay in page index (no separate index) with recency boost
3. FSRS-6 default recency model (hardcoded params, only stability configurable)
4. Debounced index scheduler over polling (matching Knowns pattern)
5. No backward compat — type param defaults to all

## Pending High Priority

- MCP E2E Integration Tests (s2ff4x)
- CLI E2E Integration Tests (7d3uvn)
- Full Workflow E2E Test (g5nm08)
- Cross-entity search E2E test (AC-12 from spec)
- vectors.bin WMV\1 type tag extension

## Gotchas

- FSRS R=S → R=0.9, not 0.5. Test assertions at day7 should check ~0.9 not ~0.5.
- .wm/memory/ path, not .knowns/memory/
- recency_model field, not fsrs_enabled
- type param defaults to "all", not backward compat "page"
