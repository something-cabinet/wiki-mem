---
title: "serde tagged enums: avoid enum-level #[allow(dead_code)], use field-level"
type: learning
status: active
tags: [rust, serde, lint, dead_code]
---

Rust's `dead_code` lint often fires on fields within serde-deserialized structs
and tagged enums because the code never reads those fields by name — serde
populates them via the derive macro.

**Wrong approach:** Adding `#[allow(dead_code)]` on the entire enum or struct
masks legitimate dead code (e.g., an unmatched variant, or a field that's truly
unused).

**Right approach:** Suppress at the individual field level:
```rust
#[derive(Deserialize)]
struct MyInput {
    used_field: String,
    #[allow(dead_code)] // populated by serde, reserved for future use
    unused_field: Option<i32>,
}
```

For tagged enums matched in handlers (`match input { ... }`), the enum and its
variants are NOT dead — only individual variant fields that aren't read need
suppression. Remove enum-level `#[allow(dead_code)]` entirely.

**Reference:**
- @wiki/tasks/review-dead-code-audit
- `apps/wm-core/src/mcp/tools/` — graph.rs, index.rs, log.rs, memory.rs, time.rs
