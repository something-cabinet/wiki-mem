---
title: 'Learning: PageType priority_rank Drift Between Enum and Search'
page_type: learning
status: draft
tags:
  - learning
  - bug
  - search
  - ranking
---
## Failure

### Pattern/Decision Rank Swap
**What went wrong:** `PageType::Pattern` had priority rank 5 in the canonical `priority_rank()` method, but search enrichment in `search/query.rs` hardcoded Pattern as rank 3 and Decision as rank 5 — swapped. This persisted for months across multiple refactors.

**Root cause:** Search enrichment duplicated the rank mapping instead of calling `priority_rank()`. When ranks were adjusted in the enum, search was never updated. There was no enforcement that the hardcoded match statement matched the canonical method.

**Time lost:** Not estimated (bug existed since initial implementation). Discovered during Sprint 4 review.

**Prevention:** Never duplicate rank/priority mappings. Always call the canonical method:
```rust
// BAD: hardcoded match statement that drifts
r.page_type_rank = match meta.page_type {
    PageType::Decision => 5,  // wrong!
    PageType::Pattern => 3,   // wrong!
    ...
};

// GOOD: single source of truth
r.page_type_rank = meta.page_type.priority_rank();
```

## Pattern

### Canonical Source for Rankings
Maintain a single `priority_rank()` method on the enum itself. All consumers call this method. If a consumer needs custom ordering, add a parameterized method rather than duplicating the match.

## Related

- @wiki/concepts/decisions/axum-over-rocket-for-tower
