---
title: 'No #[allow(...)] Attributes'
type: rule
id: wiki:rules:no-allow-attributes
status: draft
tags: [convention, rust, code-quality]
---

## Rule

Do not use `#[allow(...)]` or `#![allow(...)]` anywhere in the codebase.

## Rationale

`#[allow]` silences warnings permanently and rots. Once added, the suppressed lint never fires again — even when the original reason is gone or the code changes in ways that make the warning valid. It masks bugs.

## What to do instead

- **Fix the warning.** If clippy or rustc warns about something, fix the code.
- **Remove dead code.** If code is unused, delete it.
- **Use `#[cfg(test)]`** for test-only code that would otherwise trigger dead_code in lib builds.
- **Accept warnings in test helper files.** Shared test helpers included via `#[path = ...]` in multiple test binaries will naturally have dead_code warnings for functions not used in every binary. These warnings are acceptable — they don't fail CI and signal which helpers could be cleaned up.

## Exceptions

None. If a lint is genuinely wrong for a specific case, open a discussion before suppressing it.
