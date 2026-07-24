---
id: wiki:patterns:test-helper-path-include
title: "Test Helper Modules: #[path] Over Dead Code Suppression"
type: pattern
status: active
category: testing
rationale: "Integration test helper files accumulate unused items because each test file uses a different subset. `#[path]` includes let each test import only what it needs, eliminating dead_code warnings without suppressions."
relates_to:
  - {type: references, target: "wiki:specs:fix-rust-anti-patterns"}
---
id: wiki:patterns:test-helper-path-include

## Problem

Integration tests share helper code via `tests/helpers/mod.rs`. Each test file does `mod helpers;` and imports everything, but only uses a subset. The compiler warns about unused items. Adding `#[allow(dead_code)]` is forbidden.

## Solution

Split helpers into focused files and use `#[path]` to include only what each test needs:

```rust
// tests/helpers/cli.rs — full CLI helpers (run_cli, run_cli_with_stdin, etc.)
// tests/helpers/cli_run.rs — run_cli only (subset)
// tests/helpers/mcp.rs — full MCPClient (all methods)
// tests/helpers/mcp_basic.rs — MCPClient without list_tools
// tests/helpers/setup.rs — setup_test_project()
// tests/helpers/macros.rs — assert_success!, assert_contains!

// In each test file, include only what's needed:
#[path = "helpers/cli_run.rs"]
mod helpers;

#[path = "helpers/setup.rs"]
mod setup;
```

This compiles without warnings because each file only imports items it actually uses.

## When to Use

- Integration test directories with shared helper modules
- Any test file that uses a subset of available helpers
- When `#[allow(dead_code)]` on test helpers would violate the zero-annotation rule

## When Not to Use

- Small test files with few helpers — `mod helpers;` is simpler
- Library unit tests within `#[cfg(test)] mod tests { }` — `use super::*;` is idiomatic

## Related

- @wiki/rules/no-dead-code-clone-scanning
