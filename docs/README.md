# Wiki Memory Engine — Core Concepts

Technical documentation for key subsystems.

## Search & Ranking

- [BM25 Search Algorithm](bm25-search.md) — Field-weighted BM25 with code-aware tokenizer, score normalization, rerank boosts
- [FSRS-6 Recency Bias](fsrs6-recency-bias.md) — Forgetting curve model for task ranking, comparison vs linear/exponential
- [Cross-Entity Search](cross-entity-search.md) — Per-type BM25 indexes, type filter, RRF fusion, recency + salience boosts

## Graph

- [Edge Types & Traversal](graph-edge-types-traversal.md) — 17 edge types with priorities, typed graph structure, 4 traversal modes with depth rules

## Configuration

- [ScoringConfig](scoring-config.md) — All tunable parameters with defaults and tuning guidance

## Memory

- [Memory System](memory-system.md) — MemoryEntry format, BM25 indexing, staleness detection, salience boosts

## Platform

- [Platform Setup](platform-setup.md) — Supported MCP platforms, config formats, skill directories, agent instructions
