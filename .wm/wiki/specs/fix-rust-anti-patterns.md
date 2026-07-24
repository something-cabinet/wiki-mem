---
id: wiki:specs:fix-rust-anti-patterns
title: Fix Rust Anti-Patterns
type: spec
status: approved
tags: [spec, rust, quality, blocking-io, to-string, unwrap, deepwork]
references: "@wiki/rules/rust-anti-patterns"
---
id: wiki:specs:fix-rust-anti-patterns

## Overview

Fix actionable anti-patterns across the Rust workspace: blocking `std::fs` in async handlers, `"literal".to_string()` allocations, early `.collect::<Vec>()` in iterator chains, and `.unwrap()` calls in production code. Also: remove all `#[allow(...)]` annotations, all `//` comments, and achieve zero compiler warnings and errors.

## Requirements

- FR-1: `std::fs` → `tokio::fs` in async handlers
- FR-2: `"literal".to_string()` → `.into()` or `&str`
- FR-3: Early `.collect::<Vec>().join()` → `.fold()`
- FR-4: Production `.unwrap()` → `.expect()` or error propagation
- FR-5: Remove all `#[allow(...)]` annotations
- FR-6: Remove all `//` comments from modified files
- FR-7: Zero compiler warnings and errors across entire workspace
