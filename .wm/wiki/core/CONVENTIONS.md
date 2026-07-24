---
title: WM Conventions
type: core
id: wiki:CONVENTIONS
tags:
- conventions
- code-style
- rust
- angular
- naming
status: reviewed
relates_to:
  - {type: references, target: wiki:core:architecture}
---

# WM Conventions

## Code Style

### Rust

- **Edition**: 2021, resolver = "2"
- **Workspace deps**: Every crate MUST use `{ workspace = true }` for shared dependencies. Inline versions cause duplicate compilation.
- **Formatting**: Standard `rustfmt`. Run `cargo fmt` before committing.
- **Linting**: Run `cargo clippy` before opening a PR. Fix all warnings — they are defects.
- **Dead code**: `#[allow(dead_code)]` is never acceptable. Restructure or remove dead code. The only exception: `_schema`-prefixed fields in flatten struct patterns for MCP JSON Schema generation.

### Angular

- **Strict mode**: No `any` types. All responses are typed interfaces.
- **NgRx**: Graph data, page state, and UI state live in NgRx stores.
- **EnginePort pattern**: Components depend on an `InjectionToken<EnginePort>` interface, never on `HttpEngineService` directly. This enables testability via `MockEngineService`.
- **WASM integration**: WASM modules are lazy-loaded via dynamic `import()`, never bundled eagerly.

## File Organization

### One Type Per File

Every `.rs` file holds exactly one primary type (struct, enum, trait). The only exception is tightly coupled helper structs under 20 lines.

### Role-Based Naming

File names encode their role:

| Suffix | Purpose |
|--------|---------|
| `*_model.rs` | Pure data (structs, enums, derives) |
| `*_service.rs` | Business logic |
| `*_helper.rs` | Stateless utilities |
| `*_constant.rs` | Constants, `OnceLock`, `RustEmbed` |
| `*_repository.rs` | Data access |
| `*_proxy.rs` | Access control (lazy init, caching) |
| `*_mediator.rs` | Coordination |
| `*_builder_service.rs` | Step-by-step construction |

Files must be in role-based subdirectories: `models/`, `services/`, `helpers/`, `constants/`. Every subdirectory MUST have a `mod.rs` barrel file that re-exports all public items.

### Shared Code

If module A needs A/A.1, keep A.1 in A/. If module B also needs A.1, move A.1 to a shared location at the same level as A and B.

## Naming

- **MCP tools**: Always prefixed with `wm_` (e.g., `wm_page.create`, `wm_task.board`) to avoid collisions with host-app built-in tools.
- **Page types**: Lowercase, plural directory names: `core/`, `concepts/`, `decisions/`, `howto/`, `patterns/`, `reference/`, `specs/`, `tasks/`.
- **Page IDs**: `wiki:{type}:{name}` — e.g., `wiki:concepts:graph-architecture`, `wiki:tasks:fix-auth`.
- **Rust crates**: Lowercase kebab-case: `wm-core`, `wm-cli`, `wm-server`, `fjadra-wasm`.

## Wiki Conventions

### Page Frontmatter

Every wiki page uses YAML frontmatter:

```yaml
---
title: My Page
type: concept        # task | spec | concept | core | pattern | decision | howto | reference
status: draft        # draft | reviewed | approved | done | in-progress | todo
tags: [tag1, tag2]
relates_to:          # typed edges (optional)
  - {type: extends, target: "wiki:concepts:base-thing"}
---
```

### Cross-References

Use `@wiki/{type}/{name}` syntax:
- `@wiki/tasks/fix-auth`
- `@wiki/concepts/graph-architecture`
- `@wiki/decisions/http-wasm-seam`
- `@wiki/memory/abc123`
- `@wiki/core/conventions`

### Findings-First

Every finding from a review, audit, or analysis must have a wiki task + spec created before implementation. See @wiki/rules/findings-first-task-spec.

## MCP Tool Patterns

- All WM tools use the `wm_` prefix.
- Tool errors MUST use JSON-RPC `isError: true`, not protocol-level errors.
- `wm_help` reads schemas dynamically from `ToolRegistry`, not a hardcoded list.
- Tool registration uses `register_with_schema()` with `schemars`-derived JSON schemas.

## Memory and Knowledge

- Use `wm_memory.add(layer="project")` for repo-specific patterns and decisions.
- Use `wm_memory.add(layer="global")` for cross-project preferences.
- Use `wm_memory.add(layer="session")` for ephemeral session context.
- Memory is durable knowledge, not time-sensitive — salience boost, not recency decay.
- Never duplicate wiki page content into memory. Store a summary + reference.

## Search Conventions

- Default search mode: hybrid (RRF fusion of BM25 + cosine similarity).
- Post-RRF rerank adds heuristics: title density +0.03/word, exact title +0.15, proportional tag overlap, exact ID +0.10.
- Rerank boosts applied before RRF fusion are silently discarded — always apply after.

## Architectural Constraints

- **No Node.js or Python services** for core functionality.
- **No external databases** (turso/SQLite is fine for local state).
- **No third-party API dependencies** for core functionality.
- **WASM only for pure compute**: fs-free, tokio-free, rayon-optional, serde for I/O.
- **CLI must work offline**: engine runs in-process, never proxies through HTTP.
- **Correctness over convenience**: "over-engineered" is acceptable when it eliminates a class of bugs.

## Testing

- One test function per workflow with step comments, not fragmented tests with shared mutable state.
- For child process tests: active readiness polling with deadline, never fixed `sleep()`.
- Remove `WM_PROJECT` and similar env vars from test child process environments.
- Use `MockEngineService` (Angular) for frontend component tests.

## Git

- Do not create commits unless explicitly asked.
- Do not amend or force-push unless explicitly asked.
- Do not revert user changes you did not make.

## References

- @wiki/core:enterprise-grade — Scale targets and locked decisions
- @wiki/rules/findings-first-task-spec — Findings must become tasks + specs
- @wiki/rules/no-warnings — No compiler warnings accepted
- @wiki/core:critical-patterns — Costliest lessons learned
- @wiki/core:architecture — System architecture
