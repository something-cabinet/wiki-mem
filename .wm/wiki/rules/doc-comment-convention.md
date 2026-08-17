---
title: Doc Comment Convention
type: rule
id: wiki:rules:doc-comment-convention
status: draft
tags: [convention, documentation, rust, code-quality]
---

## Purpose

Doc comments (`///`) are API documentation — they appear in `cargo doc` output and IDE hover tooltips. They describe the contract of a public item for its callers. They are NOT a back-door for inline comments.

This rule defines when doc comments are required, what they must contain, and what they must NOT contain.

## When to Write Doc Comments

### Required

- All `pub` functions, methods, structs, enums, traits, type aliases, and constants
- All `pub(crate)` items that cross module boundaries (used outside their defining module)
- Crate-level (`//!`) and module-level documentation for public modules

### Forbidden

- Private (`fn`, not `pub`) helper functions — the name must be self-documenting
- Test functions — use descriptive test names instead
- Items where the doc comment would merely restate the function signature (e.g., `/// Returns the name` on `fn name() -> &str`)
- Any comment disguised as a doc comment: spec references, implementation notes, TODO items, or "why" explanations that belong in commit messages or wiki pages

### Optional

- `pub(crate)` items used only within their module — use judgment; if the name communicates everything, skip it

## Format

Follow RFC 505 / RFC 1574 (Rust official conventions):

### Summary Line

The first line is a single sentence in **third-person singular present indicative** form. Ends with a period.

```rust
/// Resolves import candidates for a given source file and target specifier.
pub fn resolve_import_candidates(...) -> ...
```

NOT:

```rust
/// Resolve import candidates  ← imperative mood
/// This resolves import candidates  ← starts with "This"
/// resolve_import_candidates resolves...  ← restates name
```

### Structure

After the summary line, add a blank `///` line, then optional sections **in this order**:

1. Extended description (if the summary is insufficient)
2. `# Panics` — conditions under which the function panics
3. `# Errors` — error variants returned and when
4. `# Safety` — invariants the caller must uphold (unsafe fns only)
5. `# Examples` — runnable code (for public library APIs)

Omit sections that don't apply. Most internal functions only need the summary line.

### Style Rules

- Use backticks for code: types (`Vec<T>`), functions (`resolve()`), parameters (`source_file`)
- Use intra-doc links for types: `[`CodeEdge`]` (rustdoc resolves automatically)
- Full sentences with proper punctuation
- No trailing whitespace on `///` lines
- No section headers without content beneath them

## What Goes in Doc Comments vs. Elsewhere

| Content | Where it belongs |
|---------|-----------------|
| What the function does (contract) | Doc comment |
| What the function returns | Doc comment |
| When it errors/panics | Doc comment |
| Why we chose this algorithm | Wiki decision page or commit message |
| Spec/task references (FR-2.3, AC-1) | Wiki task/spec page (not code) |
| Concurrency/thread-safety semantics | Doc comment (it's part of the contract) |
| Performance characteristics | Doc comment if callers need to know |
| Implementation details | Nowhere in code — make code self-documenting |

## Examples

### Good: Concise summary, adds value beyond the signature

```rust
/// Extracts typed cross-file code edges from a single file's AST.
///
/// Edges include `imports`, `calls`, `inherits`, and `implements` — raw
/// per-file facts. Targets are resolved against the global symbol index
/// at query time by [`GraphResolver`].
///
/// # Panics
///
/// Panics if `ext` is not a supported language extension.
pub fn extract_edges(source: &str, file: &str, ext: &str) -> Vec<CodeEdge> {
```

### Good: One-liner where that's all that's needed

```rust
/// Checks whether a type name is a language primitive that should not generate a reference edge.
fn is_primitive_type(name: &str) -> bool {
```

### Bad: Restates the signature

```rust
/// Returns the name.
pub fn name(&self) -> &str {  // ← delete the doc comment
```

### Bad: Implementation note disguised as doc comment

```rust
/// We use ArcSwap here because the graph is rebuilt on a background thread.
pub struct GraphState {  // ← this belongs in a decision page or nowhere
```

### Bad: Spec reference in doc comment

```rust
/// FR-2.5: resolves TypeScript path aliases.
pub fn resolve_ts_aliases(...) {  // ← spec refs belong in wiki, not code
```

## Enforcement

- `#![warn(missing_docs)]` on library crates (`wm-engine`, `wm-embed`, `wm-search`, `wm-code-intel`)
- Agent must check doc comments during review for compliance with this convention
- Doc comments that merely restate the function name/signature should be flagged and removed
