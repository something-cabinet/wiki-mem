---
title: 'Pattern: Cross-Crate Constants Extraction'
id: wiki:patterns:cross-crate-constants
type: pattern
relates_to:
  - {type: references, target: wiki:specs:rebuild-log-findings}
status: draft
tags: [pattern, constants, architecture, code-quality]
---

## Problem

Magic strings and numbers like `".wm"`, `"wiki"`, `4090`, `8192` are duplicated across 2-5 crates. Each crate inlines them, creating maintenance debt when a value needs to change. But creating a shared crate for every small constant is also overhead.

## Solution

**Decision rule:** When a magic value is used in 3+ crates → extract to `wm-constants` (zero-dependency shared package under `packages/`). When used in 1-2 crates → keep local in a per-crate `constants/` directory with barrel `mod.rs`.

**Per-crate layout:**
```
crate/src/
  constants/
    mod.rs        — barrel: `pub mod foo; pub mod bar;`
    foo.rs        — domain-specific constants
    bar.rs        — another domain group
```

**Shared crate (`wm-constants`):**
```
packages/wm-constants/
  Cargo.toml      — [dependencies] (empty — zero deps)
  src/
    lib.rs        — all pub const declarations
```

**Shared usage threshold:**
| Crates using the value | Where it goes | Example |
|---|---|---|
| 3+ | `wm-constants` | `".wm"` → `WM_DIR` used in 5 crates |
| 2 | Per-crate constants file | `4090` → local in wm-server + wm-cli |
| 1 | Inline near usage or local const | CLI-specific status strings |

## When to Use

- During code review, when the same literal appears in multiple crates
- When a new crate is added and needs path/configuration constants
- When an existing magic value changes and you find 5 places to update

## When Not to Use

- For values that are part of a data schema (JSON field keys, API response fields)
- For format strings passed to `format!()` (must be literals)
- For test-only constants

## Related

- @wiki/rules/no-magic-values
- @wiki/core:ARCHITECTURE
- @wiki/core:CONVENTIONS