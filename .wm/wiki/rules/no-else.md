---
title: No else — Prefer Early Return
type: rule
id: wiki:rules:no-else
status: active
tags: [rule, code-style, rust, control-flow]
---

## Rule: No `else` — Prefer Early Return

Do not use `else` blocks. Prefer early returns or guard clauses. When you can't eliminate `else` without contortions, extract a function — that tells you the original was too big.

### Why

- `else` creates nested scopes that increase cognitive load
- Early return flattens the happy path to the left margin
- Guard clauses make preconditions explicit at the top of a function
- `else` blocks are harder to refactor; extracting the `if` body often requires restructuring
- If removing `else` requires `match` on booleans or other workarounds, the real fix is a smaller function

### Allowed

**Early return (preferred):**
```rust
fn validate(x: Option<i32>) -> Result<(), Error> {
    let Some(val) = x else { return Err(Error::Missing); };
    // happy path continues here, unindented
    Ok(())
}
```

**Guard clause:**
```rust
fn process(items: &[u8]) -> Vec<u8> {
    if items.is_empty() {
        return vec![];
    }
    // rest of function
}
```

**`match` on enums (exhaustive pattern matching):**
```rust
match status {
    PageStatus::Active => handle_active(),
    PageStatus::Draft => handle_draft(),
}
```

**`if let` with `else`** — The `let ... else` pattern is the sole exception, because it binds a variable while handling the `None`/`Err` case:
```rust
let Some(val) = opt else { return None; };
```

### Forbidden

```rust
// ❌ else block
if condition {
    do_something();
} else {
    do_other();
}
```

```rust
// ❌ match on booleans to sneak around the else rule
// This signals the function should be split instead
match (a == 0, b == 0) {
    (true, true) => x,
    _ => y,
}
```

```rust
// ❌ else if chain
if a == 1 {
    x()
} else if a == 2 {
    y()
} else {
    z()
}
```

### How to decide: `match` vs early return

| Situation | Use | Why |
|-----------|-----|-----|
| Enum variants or multi-value branching | `match` | Compiler enforces coverage for enums; multi-value `match` is clearer than chained `if`/guard clauses |
| Destructuring / binding | `match` or `if let` | Pattern matching is the right tool |
| Boolean, single value, two outcomes | `if { return }` + fallthrough | Extract function if the body is complex |
| `if condition` + `else` on a single expression | `if { return }` + fallthrough, or extract function | The `else` means the scope is too wide |
| `match` on booleans to avoid `else` | ❌ *Forbidden* — extract a function instead | This is a workaround, not a use of `match` |

### Enforcement

- `rg '\}\s*else\s*\{' apps/ packages/ -g '*.rs'` — find `} else {` patterns
- `rg 'else if' apps/ packages/ -g '*.rs'` — find else-if chains
- `rg 'match\s*\((true|false|\w+\s*(==|!=|<|>)\s*\w+).*\)' apps/ packages/ -g '*.rs'` — find boolean tuple matches (workarounds)
- Pre-existing violations don't need immediate fixing, but new code must comply

### Exceptions

- Test code (test assertions sometimes need if-else for setup/teardown branches)
- Code generation output where the generator uses if-else
- The `let ... else` binding pattern is explicitly allowed