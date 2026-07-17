---
title: 'Learning: Gehenna-App Cross-Project Patterns — CDD, Error Chains, Svelte 5'
page_type: pattern
id: concepts/learning-gehenna-app-cross-project-patterns-cdd-error-chains-svelte-5
tags:
  - learning
  - cdd
  - error-handling
  - svelte5
  - conventions
---

# Learning: Gehenna-App Cross-Project Patterns

Patterns and conventions adopted from reviewing the gehenna-app codebase (Princess Connect Re:Dive clan management, Rust + SvelteKit monorepo).

## Source

- gehenna-app repo at `D:\project\gehenna-app`
- CONVENTIONS.md, Agents.md, critical-patterns.md
- Various Rust architecture learning docs

## Patterns Adopted

### 1. Compiler-Driven Development (CDD)

gehenna-app uses CDD as the Rust equivalent of TDD's Red/Green/Refactor:

```
Compile error? → Improve the code.
Compiled OK?   → Improve the model (types).
```

**Key technique: Make Invalid States Unrepresentable**
- Use newtype wrappers with constructors instead of raw primitives
- Use enums instead of strings for constrained values
- Encode state machine transitions in types (typestate pattern)

**Applied to vpp-rag:**
- [ ] ToolError now wraps I/O and serde errors with `#[source]` instead of `String`
- [ ] PageType uses enum discriminants rather than string comparisons where feasible

### 2. No "What" Comments — Extract Functions Instead

From gehenna-app CONVENTIONS Rule 6:
> If a function needs a doc comment, extract the logic into a smaller function with a descriptive name instead. Code should be self-documenting.

Comments are only allowed for:
- **Why, not what** — non-obvious business logic or workarounds
- **External references** — links to specs, issues, or docs

Alternatives to commenting:
- Vague variable → rename it
- Long function → extract smaller named functions
- Complex condition → extract a named predicate
- `TODO` → create a task ticket

**Applied to vpp-rag:**
- [ ] Engine section markers (`// ─── Write Channel ───`) kept as module-level organization
- [ ] Inline "what" comments replaced with extracted functions or removed

### 3. Typed Error Chains

gehenna-app wraps underlying errors with context-preserving error types:
```rust
pub enum RepoError {
    NotFound(String),
    Database(#[source] sea_orm::DbErr),  // preserves full context
}
```

**Applied to vpp-rag:**
- [ ] ToolError variants now wrap `io::Error` and `serde_json::Error` with `#[source]`
- [ ] Error messages include the operation and path that failed

### 4. Svelte 5 Idioms

gehenna-app enforces Svelte 5 patterns:
- `$props()` destructuring instead of `export let`
- `$derived()` for reactive derived state instead of `$:`
- `{@render children()}` with `let { children } = $props()` instead of `<slot />`
- `onclick` instead of `on:click`

**Applied to vpp-rag:**
- [ ] wm-ui audited for Svelte 4 holdovers

### 5. Guard Clauses Over if-else

From gehenna-app CONVENTIONS Rule 10:
> Prefer guard clause / early return pattern. Avoid `if-else` when the `if` branch returns early.

**Applied to vpp-rag:**
- [ ] Existing code reviewed for unnecessary `else` after early returns

### 6. Skeleton-Only for API Content

> Only content grids show skeletons during navigation. Static UI elements must remain visible.

Already mostly followed in wm-ui but worth documenting.

## Key Differences (Why Not Full Adoption)

### Service/Repository Layering

**Correction (2026-07-16):** Service and Repository are **storage-agnostic patterns** — they apply to filesystems and in-memory stores just as well as databases. The codebase already has informal repositories (`VersionStore`, `VectorStore`, `FsPageRepo`-like operations in `page.rs`).

The real question is ROI. For a single-user CLI/MCP tool:

| Worth doing | Skip |
|---|---|
| `PageRepo` trait (filesystem I/O isolation) — enables real unit tests for YAML logic | Full hexagonal / clean architecture |
| `VectorRepo` trait (turso abstraction) — already a clean struct | `SourceRepo` / `TaskRepo` / `GraphRepo` traits |
| Decompose `EngineState` God Object into component bundles | `PageService` / `SourceService` wrapper structs — free functions are idiomatic Rust |

The better long-term pattern isn't Service/Repository layering — it's **composition over the God Object**, using traits as a tool to enable that decomposition, not as a goal in itself.

### Other Differences

- gehenna-app uses `testcontainers` for integration tests. Vpp-rag uses in-process test projects (`.wm/` directories) — simpler and sufficient for MCP tools.
- gehenna-app has full Moonrepo/Turbo monorepo tooling. Vpp-rag uses Cargo workspace — simpler and sufficient.

## References

- gehenna-app CONVENTIONS.md
- gehenna-app learnings/compiler-driven-development-cdd-in-rust.md
- gehenna-app learnings/ddd-testing-strategy-for-rustseaorm.md
