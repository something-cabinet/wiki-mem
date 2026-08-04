---
type: pattern
title: 'Pattern: Repository Trait for Filesystem I/O'
page_type: pattern
id: concepts/pagerepo-trait
tags:
  - pattern
  - testing
  - filesystem
  - rust
---

## Problem

Unit testing functions that read/write files (like YAML frontmatter mutations in `page.rs`) requires spinning up an entire `EngineState` with real filesystem, embedding, and graph state. Tests are slow, fragile, and depend on disk state.

## Solution

Extract filesystem operations behind a `PageRepo` trait:

```rust
pub trait PageRepo: Send + Sync {
    fn read_to_string(&self, path: &Path) -> Result<String, io::Error>;
    fn write(&self, path: &Path, content: &[u8]) -> Result<(), io::Error>;
    fn create_dir_all(&self, path: &Path) -> Result<(), io::Error>;
    fn remove_file(&self, path: &Path) -> Result<(), io::Error>;
    fn exists(&self, path: &Path) -> bool;
}
```

Two implementations:
- **`FsPageRepo`** — production, delegates to `std::fs`
- **`InMemoryPageRepo`** — test-only, stores files in `HashMap<PathBuf, Vec<u8>>`

Public API stays backward-compatible: the existing `fn create_page(engine: &Arc<EngineState>, ...)` creates an `FsPageRepo` internally and delegates to `fn create_page_with_repo(..., repo: &dyn PageRepo)`.

## When to Use

- Any module with non-trivial file I/O that needs isolated unit tests
- YAML/JSON parsing + file write patterns (frontmatter manipulation, config updates)
- Code with complex mutating logic that currently requires a full engine bootstrap to test

## When Not to Use

- Stateless I/O (read only, no mutation) — simple `std::fs` calls are fine
- Operations where the test infra already provides temp directories (integration tests with `#[test]` + tempdir)
- Single-use scripts that won't have tests

## Related

- `src/page_repo.rs`
- `src/page.rs` (7 public functions refactored to use PageRepo)
- `InMemoryPageRepo` implements full in-memory store for isolated tests
