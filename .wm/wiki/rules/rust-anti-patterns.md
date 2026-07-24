---
id: wiki:rules:rust-anti-patterns
title: "Rust Anti-Patterns: Beyond .clone()"
type: rule
status: active
category: quality
rationale: "`.clone()` is the most visible borrow-checker bypass, but several other patterns silently harm performance, safety, or correctness. This rule catalogs them with codebase-specific baselines."
---
id: wiki:rules:rust-anti-patterns

## Rule

Minimize or eliminate the following patterns across the Rust workspace. Each has a codebase baseline and specific guidance.

---
id: wiki:rules:rust-anti-patterns

### 1. Avoid Bare `.unwrap()` and `.expect()`

**Problem:** Panics on `None`/`Err` — shifts compile-time safety to runtime crashes.

**Codebase baseline:** 243 `.unwrap()` calls, 354 `.expect()` calls

**Guidance:**
- **Infallible operations** (e.g., `.ok()?`, `&format!(...)`) — safe, but prefer `?` or `unwrap_or_default()` for clarity
- **Test code** — acceptable for test assertions where failure should panic
- **Production handlers** — use `?`, `.unwrap_or_else(|| ...)`, `.ok_or_else(|| ...)?` instead
- **Config parsing** — `.expect("...")` with a descriptive message is acceptable at startup

**Tools:**
```bash
# Find unwrap hotspots by file
rg '\.unwrap\(\)' apps/ packages/ -g '*.rs' -c | sort -t: -k2 -rn | head -20
# Find expect hotspots
rg '\.expect\(' apps/ packages/ -g '*.rs' -c | sort -t: -k2 -rn | head -20
```

**Top offender files:**
| File | `.unwrap()` | `.expect()` | Notes |
|------|-------------|-------------|-------|
| `wm-code-intel/ingest_service.rs` | 47 | — | Mostly safe (test helpers, infallible) |
| `wm-embed/src/lib.rs` | 17 | — | Error recovery needed |
| `wm-core/template_engine/mod.rs` | 14 | — | Template render errors should propagate with `?` |
| `wm-core/graph/mod.rs` | 12 | — | Some are safe, some need error propagation |

---
id: wiki:rules:rust-anti-patterns

### 2. Avoid `"literal".to_string()` and `String::from("literal")`

**Problem:** Heap-allocates a `String` from a compile-time constant. Wasteful when only reading.

**Codebase baseline:** ~45 sites in `apps/`, ~30 in `packages/`

**Guidance:**
- For static text: use `&'static str` or `Cow<'static, str>` 
- Use `String` only when you need to mutate or grow the text at runtime
- In struct fields that are always static strings: use `&'static str` or an enum instead
- `.to_string()` is acceptable when building a dynamic string from parts (format!, push_str, etc.)

**Good:**
```rust
fn help_text() -> &'static str { "Usage: wm-cli [COMMAND]" }
```

**Avoid:**
```rust
fn help_text() -> String { "Usage: wm-cli [COMMAND]".to_string() }
```

**Tools:**
```bash
rg '"[^"]*"\.to_string\(\)' apps/ -g '*.rs' -c | sort -t: -k2 -rn
```

---
id: wiki:rules:rust-anti-patterns

### 3. Avoid Early `.collect::<Vec<_>>()` in Iterator Chains

**Problem:** Breaking a lazy iterator chain with `.collect()` forces heap allocation, losing the optimization of chained `map`/`filter`/`fold`.

**Codebase baseline:** ~15 early collects in `apps/`, several in `packages/`

**Guidance:**
- Keep iterator chains lazy with `.map()`, `.filter()`, `.fold()` directly
- Only `.collect()` at the final consumer site
- Pattern `.map(...).collect::<Vec<_>>().join(...)` — use `.fold()` or `.collect::<String>()` instead

**Avoid:**
```rust
let items: Vec<_> = iter.map(|x| format!("{}", x)).collect();
let result = items.join(", ");
```

**Prefer:**
```rust
let result = iter.map(|x| format!("{}", x)).collect::<Vec<_>>().join(", ");
// Still collects but single pass — acceptable. For small iterators this is fine.
```

**Or eliminate collect entirely:**
```rust
let result = iter.fold(String::new(), |acc, x| {
    if acc.is_empty() { format!("{}", x) } else { format!("{}, {}", acc, x) }
});
```

**Tools:**
```bash
rg '\.collect::<Vec' apps/ packages/ -g '*.rs'
```

---
id: wiki:rules:rust-anti-patterns

### 4. Avoid Blocking I/O in Async Context

**Problem:** `std::fs` operations inside async handlers (tokio) block the thread-pool thread, stalling other tasks.

**Codebase baseline:** `std::fs::read_to_string`, `std::fs::read_dir`, `std::fs::write` used directly in several MCP tool handlers

**Guidance:**
- In tokio async handlers, use `tokio::fs` instead of `std::fs`
- For one-off reads in non-critical paths, `std::fs` is acceptable but prefer `tokio::fs::read_to_string`
- For heavy operations (directory walks, large file reads), use `tokio::task::spawn_blocking`

**Focus files:** `apps/wm-core/src/mcp/tools/lsp.rs`, `doc.rs`, `time.rs`, `model.rs`

**Tools:**
```bash
rg 'std::fs::' apps/wm-core/src/mcp/ -g '*.rs'
```

---
id: wiki:rules:rust-anti-patterns

### 5. Avoid `unsafe` Unless Strictly Necessary

**Codebase baseline:** ZERO `unsafe` blocks — ✅ clean

This codebase has no `unsafe` usage outside test regex patterns. Maintain this.

---
id: wiki:rules:rust-anti-patterns

### 6. Minimize Unnecessary Allocation (Box, Heap)

**Problem:** Boxing or heap-allocating data that could live on the stack causes cache misses and fragmentation.

**Guidance:**
- Prefer `enum` over `Box<dyn Trait>` for bounded variant sets
- Prefer `Vec` on the stack (it owns heap data but the pointer/metadata is on the stack)
- `Arc` is already used appropriately for shared engine state
- Avoid `Box<dyn Fn>` where a generic or enum dispatch works

---
id: wiki:rules:rust-anti-patterns

### 7. Prefer `?` Over Panic-Prone Shortcuts

**Problem:** `.unwrap()`, `.expect()`, index `[i]`, and `.unwrap_or()` on fallible operations mask errors.

**Guidance:** Every `.unwrap()` in production code should have a code review justification. The `?` operator is the default for `Result`/`Option` propagation.

---
id: wiki:rules:rust-anti-patterns

## Enforcement

- Code review must check for these patterns in new/modified code
- `rg '\.unwrap\(\)'` and `rg '\.expect\('` counts should trend down over time
- `rg '"[^"]*"\.to_string\(\)'` — reduce where the string is consumed as `&str`
- `rg 'std::fs::' apps/wm-core/src/mcp/` — flag blocking I/O in async handlers
- No new `unsafe` blocks without explicit approval

## Baseline (Jul 2026)

| Pattern | Count | Trend target |
|---------|-------|-------------|
| `.unwrap()` | 243 | ⬇ Reduce in production code |
| `.expect()` | 354 | ⬇ Reduce in production code |
| `"literal".to_string()` | ~75 | ⬇ 50% (convert to `&str`) |
| Early `.collect::<Vec<_>>()` | ~15 | ⬇ Consolidate chains |
| `unsafe` blocks | 0 | ✅ Maintain |
| Blocking fs in async | ~20 | ⬇ Migrate to tokio::fs |
