---
id: c5u8n0
title: ToolError typed error chaining
layer: project
category: pattern
tags:
  - error-handling
  - rust
  - toolerror
  - logging
createdAt: '2026-07-07T10:34:48.523Z'
updatedAt: '2026-07-07T10:34:48.523Z'
---

ToolError should carry `source: Option<Box<dyn StdError>>` to preserve error context. Use specific constructors: `io_error(op, path, err)` for I/O failures, `serde_error(op, err)` for serialization, `lock_poisoned(resource)` for mutex poison. Removed PartialEq+Eq derives to support error chaining. This allows `Display` to show the full error chain and `Error::source()` to return the underlying error.
