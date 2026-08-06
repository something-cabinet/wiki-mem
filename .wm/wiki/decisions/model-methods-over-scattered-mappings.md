---
title: 'Decision: Model Methods Over Scattered Mapping Functions'
type: decision
id: wiki:decisions:model-methods-over-scattered-mappings
relates_to:
  - {type: references, target: wiki:tasks:3db0ea}
---
id: wiki:decisions:model-methods-over-scattered-mappings

---
id: wiki:decisions:model-methods-over-scattered-mappings
title: Decision: Model Methods Over Scattered Mapping Functions
type: decision
status: approved
tags: [decision, architecture, rust, serde, enum]
---
id: wiki:decisions:model-methods-over-scattered-mappings

## Context

EdgeType serde/string mapping was duplicated across 3 modules:
- `relation_helper.rs` had `edge_type_to_yaml_str()` + `parse_edge_type_flexible()`
- `parser/mod.rs` had `parse_edge_type()` (different alias set, `Result` return)
- Validator and graph code call these functions with no single source of truth

Adding a new EdgeType variant required updating all 3 locations. The alias sets had already drifted (e.g., `parse_edge_type` didn't accept `"example-of"` kebab-case).

## Decision

Move string mapping methods directly onto the model:

```rust
impl EdgeType {
    pub fn as_yaml_str(&self) -> &str { ... }
    pub fn from_str_flexible(s: &str) -> Self { ... }
}
```

Remove the scattered standalone functions and update call sites to use `EdgeType::from_str_flexible()` and `EdgeType::as_yaml_str()`.

## Rationale

- **Single source of truth**: one match block, not 3 that can drift
- **Discoverability**: `EdgeType::` autocomplete surfaces both methods
- **No import overhead**: the type is already imported everywhere it's used
- **Cleaner call sites**: `EdgeType::from_str_flexible(s)` vs `parse_edge_type_flexible(s).unwrap_or(...)`

## Consequences

- Any code that needs EdgeType string mapping now goes through the model
- The old standalone functions were removed; existing call sites updated
- No alias set drift — all parsers share one match block
- `Result` return was unnecessary (never returned Err) — flattened to direct return

## Related
- @wiki/tasks/3db0ea