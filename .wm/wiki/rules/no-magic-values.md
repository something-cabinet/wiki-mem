---
title: No Magic Values — Use Enums and Constants
type: rule
id: wiki:rules:no-magic-values
status: active
tags: [rule, code-style, rust, quality]
---

## Rule: No Magic Values — Use Enums and Constants

Every literal string, number, or boolean embedded directly in logic must have a named binding. If the value is one of a closed set, it must be an enum. If it's a single constant, it must be a `const` or `static`.

### Why

- Magic values hide intent — `if x == 3` means nothing; `if x == MAX_RETRIES` means something
- Duplicated magic values diverge silently — change one site and miss another
- Enums make invalid states unrepresentable — the compiler enforces coverage
- Named constants enable single-point changes across the codebase
- Code review can't catch semantic errors in bare literals

### Allowed

**Enum for closed sets:**
```rust
#[derive(Clone, Copy)]
enum HttpStatus {
    Ok,
    NotFound,
    InternalError,
}
```

**Const for single values:**
```rust
const MAX_RETRIES: u32 = 3;
const DEFAULT_PORT: u16 = 4090;
const CONFIG_FILE: &str = ".wm/config.json";
```

**Newtype wrapper for typed scalars:**
```rust
struct Port(u16);
struct RetryCount(u32);
```

### Forbidden

```rust
// ❌ magic number
if retries > 3 { ... }

// ❌ magic string
let path = ".wm/config.json";

// ❌ magic boolean parameter
process(true);

// ❌ inline literal in comparison
if status_code == 404 { ... }

// ❌ bare number in array/vec
let ports = vec![4090, 3000, 8080];

// ❌ magic string as key
map.get("status");
```

### Enforcement

- `rg '\b\d{4,}\b' apps/ -g '*.rs'` — flag numbers >= 1000 that might be ports, limits, sizes
- `rg '\"[a-z./]+\"' apps/ -g '*.rs'` — flag string literals that look like paths or keys
- Code review must flag bare literals in comparisons, assignments, and function arguments
- Pre-existing magic values don't need immediate fixing, but new code must comply

### Exceptions

- `0` and `1` in index/math contexts — `array[i + 1]`, `counter += 1`, modulo operations
- Test assertions where the literal IS the expected value being tested
- Build/compile-time constants (`env!("CARGO_PKG_VERSION")`, `include_str!`)
- Format strings in `format!`/`println!` — the literal is the template, not a magic value
- Enum discriminant values where the number is part of the protocol spec (define as `const` next to the enum instead)