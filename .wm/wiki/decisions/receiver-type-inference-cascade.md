---
title: 'Decision: Receiver-Type Inference via Cascading Heuristics'
type: decision
id: wiki:decisions:receiver-type-inference-cascade
status: draft
tags: [decision, code-intel, resolution]
relates_to:
  - {type: references, target: wiki:tasks:code-edge-resolution-05-infer-receiver-types-in-the-global-resolution-pass}
---

## Context

Task 04 added receiver expressions to code edges, but without type inference the receiver is just a string (`"x"`, `"self"`, `"Foo"`) — bare-name lookup still sees all symbols named `method` across all files. The question was where and how to infer receiver types without violating NFR-2.1 (no LLM, no network, no subprocess).

## Decision

Implement a cascading inference strategy inside the existing `resolve_symbol_edge` function, using only data already available in the code index. Three sources tried in order:

1. **self/this/Self** — receiver type equals the enclosing impl or class (from `source_symbol`)
2. **Direct type prefix** — receiver IS a known type name in the symbol index (e.g. `Foo::assoc()`)
3. **Constructor binding** — if the same function scope has exactly one `Type::new()` call where `Type` is known, assume the binding holds that type

Fall back to the original candidate set when no source applies. No data beyond what extraction already provides.

## Rationale

Alternatives considered:
- **Full binding-type analysis** (parse let-type annotations, function parameters) — requires AST re-parsing at resolution time, which makes resolution non-incremental and couples it to the parser. Deferred.
- **LSP verification pass** — `packages/wm-lsp` resolves types exactly, but requires subprocess and violates NFR-2.1. Tracked as an open question for later opt-in.
- **Do nothing** — under D3, multi-candidate edges are dropped, so without inference the 12,906 receiver-bearing edges would mostly produce nothing. The ~4x visibility gain from task 04 would be wasted.

The chosen approach covers the high-confidence cases (self/this is dominant in Rust, type prefixes are all path calls, constructors are the dominant binding source) while being zero-cost at query time (runs once at index time per raw edge).

## Consequences

- Typed parameters and cross-file return types remain unresolved — AC-4 and full AC-5 are honestly marked as gaps
- The heuristic is wrong when multiple `::new()` calls exist in one scope — it falls back rather than guessing
- Adding a new inference source requires only adding a branch to the cascading match in `resolve_symbol_edge`

## Related

- @wiki/tasks/code-edge-resolution-05-infer-receiver-types-in-the-global-resolution-pass
- @wiki/specs/code-edge-resolution