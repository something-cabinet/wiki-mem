---
id: wiki:patterns:arc-vec-section-corpus
title: "Arc<Vec<T>> Clone-on-Write for Section Corpus"
type: pattern
status: active
category: performance
rationale: "Cloning an entire `Vec<SectionDoc>` (O(n) allocation) on every graph rebuild is the single most expensive clone in the codebase. Using `Arc::make_mut()` provides clone-on-write semantics."
see: "@wiki/rules/no-dead-code-clone-scanning"
relates_to:
  - {type: references, target: "wiki:specs:dead-code-clone-cleanup"}
  - {type: references, target: "wiki:specs:fix-clone-calls"}
  - {type: example_of, target: "wiki:concepts:arc-swap-try-unwrap-race"}
---
id: wiki:patterns:arc-vec-section-corpus

## Problem

In `graph/mod.rs`, the section corpus is stored as `Arc<Vec<SectionDoc>>`. Two code paths clone the entire vector:

```rust
// graph/mod.rs:244
let mut corpus: Vec<SectionDoc> = (*existing).clone();

// graph/mod.rs:297
let mut corpus: Vec<SectionDoc> = (*existing).clone();
```

Each call is O(n) with heap allocation. When the corpus has thousands of entries, this is the dominant cost in graph rebuild.

## Solution

Use `ArcSwap::rcu()` (read-copy-update) instead of `load_full().clone()` + `store()`:

```rust
// Before: always clones the entire Vec
let existing = engine.section_corpus.load_full();
let mut corpus: Vec<SectionDoc> = (*existing).clone();
corpus.retain(|s| s.page_id != page_id);
corpus.extend(sections);
engine.section_corpus.store(Arc::new(corpus));

// After: rcu clones only under contention
let pid = page_id.clone();
engine.section_corpus.rcu(|old| {
    let mut c: Vec<SectionDoc> = (**old).clone();
    c.retain(|s| s.page_id != pid);
    c.extend(sections.clone());
    Arc::new(c)
});
```

`rcu()` runs a CAS retry loop internally. Under no contention it clones once (same as the old code). The key win: it never exposes an intermediate empty state and handles concurrent readers correctly.

### Why not `Arc::make_mut()`?

`Arc::make_mut()` requires exclusive ownership of the `Arc`. With `ArcSwap`, every `load_full()` increments the refcount, so the fetched `Arc` is never exclusive. `make_mut()` would clone every time — identical cost to the naive approach.

### Why not `swap` + `try_unwrap`?

Early versions of this pattern attempted to `swap` the `ArcSwap` with an empty `Vec`, `try_unwrap` the old `Arc`, and mutate in place. This had a **data-loss race**: if any reader held a strong ref to the old `Arc` (e.g., a concurrent search), `try_unwrap` failed and `unwrap_or_default()` produced an empty `Vec`, silently wiping the corpus. `rcu()` avoids this by cloning under contention.

## When to Use

- `Arc<T>` behind `ArcSwap` that needs occasional mutation
- Read-heavy, write-rare workloads
- When correctness under concurrent readers is critical

## When Not to Use

- For `Arc<T>` that you own exclusively (use `Arc::make_mut()`)
- For small structs where clone cost is negligible
- When the closure is expensive (rcu retries on CAS failure)

## Example

```rust
// Correct pattern: rcu for ArcSwap<Vec<T>>
engine.section_corpus.rcu(|old| {
    let mut c: Vec<SectionDoc> = (**old).clone();
    c.retain(|s| s.page_id != page_id);
    Arc::new(c)
});
```
