---
title: 'Borrow Errors: Order, Area, Layout'
type: pattern
id: "wiki:patterns:borrow-errors-order-area-layout"
status: draft
tags: [rust, ownership, architecture, learning]
---

schema_version: 1
state: |-
  ## Problem

  A refactor that compiled a minute ago now fails with `cannot borrow *self as mutable more than once at a time`, or `cannot borrow *self as mutable because it is also borrowed as immutable`. Nothing semantic changed - only the shape of the code. This is what people mean by fighting the borrow checker.

  ## Why it happens

  Rust eliminates *aliased mutation*. The rule is Mutable XOR Shared: many readers, or one writer, never both at once, enforced statically. Two mechanisms explain almost every surprise.

  **Partial borrows.** For structs and tuples the compiler tracks fields *separately*, to arbitrary nesting depth - but only when fields are accessed directly inside a single function. A method taking `&mut self` borrows the **whole** struct, because the compiler does not look inside the method body. Extracting one line into a helper therefore *widens* the borrow from a single field to the entire value. That is exactly how a safe-looking refactor breaks, and it is the failure this pattern is named for.

  **Reborrowing.** `&mut *b` reborrows explicitly; passing a `&mut` to a function reborrows implicitly, so the original reference survives the call. A *shared* reborrow (`&*b`, `c = &b`, or a coercion to `&T`) makes the mutable reference behave like a shared one while the alias is live, and exclusivity resumes when the alias dies. `&mut T` is move-only - it is neither `Copy` nor `Clone`, because duplicating it would forge exclusivity.

  ## Solution: three approaches

  Apply the first one that works.

  ### 1. Change the order of accesses

  End the conflicting borrow before taking exclusive access: collect into an owned buffer, drop the iterator, then mutate. Cheapest fix, and correct whenever the access ranges can genuinely be sequenced.

  ### 2. Change the area covered by the references

  Narrow `&mut self` to only the fields actually needed - pass `&mut self.matches` and `&self.text` instead of `&mut self`. Correct while the helper touches a small, stable set of fields, but it degrades into parameter sprawl as fields accumulate.

  ### 3. Change the layout of the data to match access patterns

  Group the fields that are accessed together into their own type, so that one reference covers exactly the group. `Document { text, matches, thumbnail }` becomes `Document { text, view: View { matches, thumbnail } }`, and the helper takes `&View` plus `&str`.

  This is the only approach that *removes* the conflict instead of relocating it, and the only one that keeps working as the struct grows. Parameter sprawl from approach 2 is the signal that the layout is wrong.

  ## Selection

  | Situation | Approach | Why |
  |---|---|---|
  | Access ranges can be sequenced; hot path; restructuring too costly | Order | Smallest diff, no type churn |
  | Helper needs one or two fields, stable over time | Area | Explicit and local |
  | Data outlives the function, or fields are always touched together | Layout | Removes the conflict and survives growth |
  | Many fields needed, in two groups | Layout | Area degenerates into parameter sprawl |

  ## In this repository

  - **Layout at architecture level** - `apps/wm-core/src/engine/engine_state_mediator.rs:22-27`: `graph`, `section_corpus`, `bm25_index` and `vector_registry` are four `ArcSwap` fields that must be swapped together, so the snapshot type groups what changes together. `packages/wm-embed/src/services/vector_store_service.rs:14-48` does the same for the entry map plus the `[u8;32]` hash map, whose consistency depends on being replaced as one unit.
  - **Order** - `apps/wm-core/src/graph/mod.rs:277,354,404` and `apps/wm-core/src/page/services/page_crud_service.rs:110` mutate through `.rcu()`, where the loaded snapshot has to die before the write. The same discipline as refreshing derived state only after a successful write.
  - **The exact trap** - `apps/wm-core/src/page/services/page_update_builder_service.rs:15` (`PAGE_WRITE_LOCKS`), `packages/wm-code-intel/src/helpers/parser_helper.rs:13-18` (`OnceLock<Mutex<Parser>>`, because `Parser` is not `Sync`), and `packages/wm-embed/src/services/onnx/mod.rs:101-104` (thread-local `RefCell<HashMap<String, Session>>`). Extracting a `&mut self` helper in any of these reproduces the case verbatim. The `RefCell` variant fails at **runtime** with `BorrowMutError`, not at compile time.
  - **Area** - `record_format_hint(page_type, body)` takes exactly what it needs, in contrast to threading `&Arc<EngineState>` through most of the codebase.

  ## Constraints in this repository

  - `no-comments-in-code` means a note like `keep this borrow alive` cannot be written down. That is a structural argument for approach 3: the reasoning has to live in the shape of the data.
  - `no-compensating-layers` applies to ownership as well. Reaching for `ArcSwap`, `RefCell` or `.clone()` to silence a borrow error that a layout change would fix is a compensating layer - fix the layer that is wrong.
  - Prefer `Arc::clone` (a refcount bump) over deep clones; cloning the full section corpus is never acceptable. See `@wiki/patterns/arc-vec-section-corpus`.
  - `dbg!` is a warning in this workspace (`dbg_macro = "warn"`) and warnings are defects, so borrow investigations use `tracing` instead.

  ## Related

  - `@wiki/patterns/arc-swap-graph`
  - `@wiki/patterns/arcswap-copy-on-write-incremental`
  - `@wiki/concepts/arc-swap-try-unwrap-race`
  - `@wiki/concepts/rust-learning-research-sources`
  - `@wiki/howto/rust-inspect-variables-and-layout`

  ## Source

  RustCurious episode 10, "Three Ways to Fix Any Borrowing Error" - transcript-derived and mapped onto this repository. See `@wiki/concepts/rust-learning-research-sources`.
questions:
  - id: problem_kind
    type: choice
    instructions: What kind of problem does this pattern solve?
    options:
    - architecture
    - api-design
    - data-model
    - error-handling
    - performance
    - testing
    - ui
    - tooling
    - workflow
  - id: preconditions_required
    type: noul
    instructions: This pattern requires specific preconditions to be met.
  - id: complexity
    type: score
    instructions: How complex is applying this pattern?
    levels:
    - trivial
    - simple
    - moderate
    - complex
  - id: language_specific
    type: noul
    instructions: This pattern is specific to a programming language.
answers:
  problem_kind: architecture
  preconditions_required: true
  complexity: moderate
  language_specific: true
