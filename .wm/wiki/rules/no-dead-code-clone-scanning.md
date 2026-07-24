---
id: wiki:rules:no-dead-code-clone-scanning
title: "No Dead Code & Clone Optimization"
type: rule
status: active
category: quality
rationale: "Dead code creates maintenance burden and obscures real issues. Unnecessary `Clone` derives and `.clone()` calls waste CPU and memory. Both compound over time and normalize mediocrity."
---
id: wiki:rules:no-dead-code-clone-scanning

## Rule

1. **No `#[allow(dead_code)]`** — not on modules, not on items, not on fields. Dead code must be removed or restructured, never suppressed.

2. **No `#[allow(unused_*)]`** — unused imports must be removed; unused variables must be prefixed with `_`; unused `mut` must be dropped.

3. **No blanket `#[allow(clippy::*)]`** — each suppression must name the specific lint and include a comment explaining why the lint is wrong for this specific case, or the underlying issue must be fixed.

4. **Minimize `Clone` surface** — `#[derive(Clone)]` must be justified. Only derive Clone on types that are:
   - Passed across async task boundaries (tokio::spawn, thread::spawn)
   - Stored in `Arc`-wrapped configs or shared state that genuinely needs ownership
   - Cloned in hot paths where borrowing isn't structurally possible
   
   Types used purely as DTOs (serialization/deserialization) should not derive Clone unless there is an actual code path that clones them. Comment the justification.

5. **Audit `.clone()` calls** — every `.clone()` call is a maintenance signal. Prefer borrows, `Arc`, or restructuring over cloning. Hot-path clones and clones of large structures must be justified.

### `.clone()` Call Categories

| Pattern | Cost | Verdict |
|---------|------|---------|
| `Arc::clone()` / `arc.clone()` | Refcount bump only | ✅ Acceptable |
| `String::clone()`, `id.clone()`, `path.clone()` | Heap alloc + copy | ⚠️ Necessary for async ownership, but prefer `Arc<str>` or sharing where possible |
| `fm.field.clone()` in parser extraction (O(n) small fields) | Cheap per-call, compounds | ⚠️ Replace with `take()` / `mem::take()` when source is consumed afterward |
| `Vec<SectionDoc>::clone()` (full corpus) | O(n) heap alloc + element clones | ❌ **Never** — use `Arc::make_mut()` or clone-on-write instead |
| `cfg.field.clone()` behind `RwLock` | Single field copy | ⚠️ Acceptable for small fields, but batch into one read if multiple fields needed |
| `HashMap/Vec/BTreeMap::clone()` | O(n) heap alloc | ❌ **Heavy** — prefer `Arc`, references, or structural sharing |

## Enforcement

- Run `cargo clippy --workspace` before commits — it catches unnecessary clones, derives, and borrows
- `rg '#\[allow\(dead_code'` to find dead-code suppressions
- `rg '#\[allow\(unused'` to find unused suppression
- `rg '#\[allow\(clippy'` to find lint suppressions
- `rg '#\[derive\(.*Clone'` to list all Clone derives — review each for necessity
- `rg '\.clone\(\)' | wc -l` to track clone count trends
- Baseline: ~310 `.clone()` calls (Jul 2026). Track trends over time.

## Exceptions

- **MCP JSON Schema generation**: fields that exist solely for schema generation may use the flatten struct pattern with `_schema` prefix (see WIKI-MEM.md §Enterprise Correctness). These MUST use `#[serde(flatten)]` + `_schema` naming + `..` in match arms.
- **Feature-gated dead code**: code behind `#[cfg(feature = "...")]` that is dead in the current build must gate the usage, not suppress the warning.
- **Trait signature parameters**: `#[allow(unused_variables)]` is acceptable for trait/interface parameters that the signature requires but the implementation doesn't use, with a comment explaining why.

## Rationale

Compiler warnings are defects. Every warning accepted is a bug waiting to happen. Clone-derived costs compound silently — each unnecessary `Clone` derive propagates through the type graph, forcing downstream types to also be Clone. Every `.clone()` call incurs an allocation or copy that could be a borrow. Suppress nothing, fix everything.
