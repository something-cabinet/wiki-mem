---
id: wiki:concepts:arc-swap-try-unwrap-race
title: "Failure: ArcSwap swap() + try_unwrap() Data Race"
type: concept
status: active
tags: [failure, concurrency, arc-swap, data-loss]
rationale: "Using swap() + try_unwrap() on ArcSwap to avoid a Vec clone introduced a silent data-loss race under concurrent readers."
relates_to:
  - {type: references, target: "wiki:specs:fix-clone-calls"}
  - {type: example_of, target: "wiki:patterns:arc-vec-section-corpus"}
---
id: wiki:concepts:arc-swap-try-unwrap-race

## What went wrong

Replacing `ArcSwap::load_full().clone()` + `store()` with `swap()` + `try_unwrap()` to avoid the Vec clone. The intent was to get exclusive ownership of the Arc, mutate in place, and store back — zero copy.

## Root cause

`ArcSwap::swap()` atomically replaces the stored Arc with a new one and returns the old Arc. However, `Arc::try_unwrap()` only succeeds when the refcount is exactly 1 — meaning no other thread holds a reference. Readers do hold references: `.load()` returns an `Arc` guard, and concurrent search operations keep those guards alive during the swap window.

When `try_unwrap()` fails (refcount > 1), `unwrap_or_default()` returns an **empty Vec**. This empty Vec is then stored back into the ArcSwap, silently wiping the entire section corpus until the next full reindex. Concurrent search queries return zero results for all documents not in the current file's sections.

## Prevention

Use `ArcSwap::rcu()` (read-copy-update) instead. `rcu()` runs a CAS retry loop:
- It reads the current value, clones it, applies the mutation, and attempts a CAS
- If the CAS fails (another writer interleaved), it retries with the new value
- Never exposes an intermediate empty state
- Handles concurrent readers correctly

```rust
// Wrong — data loss under contention
let corpus = engine.section_corpus.swap(Arc::new(Vec::new()));
let mut corpus = Arc::try_unwrap(corpus).unwrap_or_default();
corpus.retain(|s| s.page_id != page_id);
engine.section_corpus.store(Arc::new(corpus));

// Correct — safe under concurrent readers
engine.section_corpus.rcu(|old| {
    let mut c: Vec<SectionDoc> = (**old).clone();
    c.retain(|s| s.page_id != pid);
    Arc::new(c)
});
```

## Time lost

~30 minutes to identify the race, reproduce the failure mode, and apply the fix.

## Related

- @wiki/patterns/arc-vec-section-corpus
- @wiki/specs/dead-code-clone-cleanup
