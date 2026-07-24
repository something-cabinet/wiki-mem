---
id: wiki:patterns:crate-extraction-with-backward-compat
title: Pattern: Crate Extraction with Backward Compat
type: pattern
tags: [pattern, refactor, packages, workspace]
---
id: wiki:patterns:crate-extraction-with-backward-compat

## Problem

How to extract modules from a monolithic crate into standalone packages without breaking existing imports across the workspace.

## Solution

Use `pub use` re-exports in the original crate's `lib.rs`:

```rust
// In apps/wm-core/src/lib.rs
pub use wm_engine as engine;    // ⇐ was pub mod engine;
pub use wm_embed as embed;      // ⇐ was pub mod embed;
pub use wm_error as error;      // ⇐ was pub mod error;
```

This makes `wm_core::engine::EdgeType` resolve to `wm_engine::EdgeType` transparently. All existing imports continue to work without changes.

## Steps

1. Create the new package under `packages/<name>/`
2. Move source files (git works best if you `git mv` for rename detection)
3. Fix imports: `crate::foo::` → `crate::` (within the package) or `wm_foo::` (external deps)
4. Rename `mod.rs` → `lib.rs` (packages require lib.rs)
5. Add to `[workspace] members` in root `Cargo.toml`
6. Add dependency in the original crate's `Cargo.toml`
7. Replace `pub mod foo;` with `pub use wm_foo as foo;` in original `lib.rs`
8. Move feature flags and optional deps to the new package

## Feature Flag Propagation

When the extracted module has feature-gated dependencies, those deps move WITH the module:

```toml
# wm-core/Cargo.toml (before)
embed = ["dep:ort", "dep:tokenizers"]
ort = { version = "2", optional = true }

# wm-core/Cargo.toml (after)
embed = ["wm-embed/onnx"]

# wm-embed/Cargo.toml
[features]
onnx = ["dep:ort", "dep:tokenizers"]
[dependencies]
ort = { version = "2", optional = true }
```

## When to Use

- Any monorepo where a crate exceeds ~3K lines
- When modules have zero or minimal internal dependencies
- When the module's functionality is useful outside the original crate

## When Not to Use

- Circular dependencies between packages (restructure first)
- Modules tightly coupled to the crate's data model
- Feature flags with complex dependency trees (test first)

## Related

- `packages/wm-engine/` — largest extraction, ~500 lines of types
- `packages/wm-code-intel/` — feature-gated extraction with tree-sitter deps
- `packages/wm-embed/` — onnx feature propagation