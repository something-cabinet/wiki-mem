---
title: Safe rustdoc-strip sweep
type: memory
tags: [comments, sweep, rust, schemas]
status: active
---

Ban-all-comments sweep: strip only lines whose first non-whitespace token is `///` or `//!` — never bare `//` (36 string/URL hazard lines like `"Read(//**)"`, `https://`). Preserve `#[schemars(description=...)]` (MCP schemas come from attributes, not `///`) and string literals. 1406 lines/113 files removed, zero warnings. Full: @doc/patterns/safe-rustdoc-strip-sweep