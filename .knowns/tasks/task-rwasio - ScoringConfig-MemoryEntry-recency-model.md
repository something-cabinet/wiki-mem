---
id: rwasio
title: ScoringConfig + MemoryEntry + recency model
status: done
priority: high
labels:
  - from-spec
  - go-mode
createdAt: '2026-07-07T04:51:10.210Z'
updatedAt: '2026-07-07T04:54:41.093Z'
timeSpent: 0
---
# ScoringConfig + MemoryEntry + recency model

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add foundation data structures: 
1. `ScoringConfig` struct in config.rs with: recency_model, recency_stability_days, field_weights, memory_salience_boost/clamp, graph depth parameters, debounce_ms, retrieve_token_budget
2. `MemoryEntry` struct in engine.rs with: id, title, content, tags, created_at, updated_at
3. Recency bias function in search.rs: fn recency_boost supporting fsrs/linear/exponential/none models
4. Wire into ProjectConfig: extend SearchConfig with ScoringConfig field
5. Unit tests for recency_boost
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

