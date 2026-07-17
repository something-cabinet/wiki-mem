---
title: "Rule as a First-Class Page Type"
page_type: spec
status: draft
tags: [spec, rule, type-system, page-type]
---

## Overview

Add `PageType::Rule` as a first-class entity in the wiki type system, alongside Spec, Decision, Pattern, Concept, Task, and Memory. A Rule is a strict, non-negotiable coding constraint that AI agents self-enforce on every action.

## Loading Strategy

Rules are loaded via **`WIKI-MEM.md`** — the file every agent reads automatically at session start, no init required.

**The chain:**
1. Agent starts → reads `WIKI-MEM.md` at repo root (automatic in every agent platform)
2. `WIKI-MEM.md` has a `## Rules` section instructing: *"Load active rules via `wm_page.list({"type": "rule", "status": "active"})`"*
3. Agent calls `wm_page.list(...)` → receives all active Rule pages with full metadata
4. Rules stay in agent context for the entire session, across every tool call

No new MCP tool needed. Existing page listing with a `type` filter handles it. No wm-init dependency. Rules are always in context.

## Requirements

### Functional Requirements

- **FR-1**: `PageType::Rule` variant exists in the `PageType` enum with `as_str()`, `allowed_statuses()`, `priority_rank()`
- **FR-2**: `RuleData` struct with typed fields: `category` (RuleCategory enum), `rationale`, `example`, `anti_pattern`
- **FR-3**: `RuleCategory` enum with 9 variants: `Naming`, `Branching`, `Design`, `ModuleStructure`, `ErrorHandling`, `DataModeling`, `Concurrency`, `Testing`, `Operational`
- **FR-4**: `rule_data: Option<RuleData>` field on `WikiPageMeta`
- **FR-5**: `Rule { meta, data }` variant on `Page` enum with `From` impls (both directions)
- **FR-6**: Frontmatter parsing/serialization for rule pages in `parser/`
- **FR-7**: Graph inference: files at `.wm/wiki/rules/*.md` are parsed as `PageType::Rule`
- **FR-8**: `rules/` subdirectory added to wiki directory for rule storage
- **FR-9**: Existing `wm_page.list` filter supports `"type": "rule"` — no separate tool needed
- **FR-10**: `WIKI-MEM.md` updated with `## Rules` section instructing agents to call `wm_page.list({"type": "rule", "status": "active"})`

### Non-Functional Requirements

- Rules are always strict — no severity field, no optional enforcement
- Rules are agent-enforced — no lint integration, no CI gate needed
- Rule loading is additive — no gate, no failure on zero matches

## Acceptance Criteria

- [ ] AC-1: `PageType::Rule` compiles, serializes, deserializes
- [ ] AC-2: `RuleData` and `RuleCategory` defined and exported from `engine::page_data`
- [ ] AC-3: Rule pages can be created, read, updated via existing page CRUD tools
- [ ] AC-4: Rule pages appear in `wm_page.list({"type": "rule"})` results
- [ ] AC-5: `wm_page.list({"type": "rule", "status": "active"})` returns only active rules
- [ ] AC-6: Files at `.wm/wiki/rules/*.md` auto-resolve to `PageType::Rule`
- [ ] AC-7: `WIKI-MEM.md` contains `## Rules` section with `wm_page.list` instruction
- [ ] AC-8: `RuleCategory::Operational` variant exists for runtime safety rules

## Rule Data Model

```rust
pub enum RuleCategory {
    Naming,
    Branching,
    Design,
    ModuleStructure,
    ErrorHandling,
    DataModeling,
    Concurrency,
    Testing,
    Operational,
}

pub struct RuleData {
    pub category: RuleCategory,
    pub rationale: String,
    pub example: Option<String>,
    pub anti_pattern: Option<String>,
}
```

## Example Rule File

```markdown
---
title: "Kill Node Process by PID"
type: rule
status: active
category: operational
rationale: "Kimaki and OpenCode run on Node — killing by name kills the wrong process"
example: "taskkill /PID <pid> /F (Windows) | kill -9 <pid> (Linux)"
anti_pattern: "taskkill /F /IM node.exe or killall node"
---
```

## Technical Breakdown

| File | Change | Est. Lines |
|------|--------|-----------|
| `engine/page_type.rs` | Add `Rule` variant + `as_str`/`allowed_statuses`/`priority_rank` | +8 |
| `engine/page_data/rule.rs` | New: `RuleCategory` enum + `RuleData` struct | +16 |
| `engine/page_data/mod.rs` | `pub mod rule; pub use rule::*;` | +2 |
| `engine/page/meta.rs` | `rule_data: Option<RuleData>` field | +1 |
| `engine/page/page_enum.rs` | `Rule { meta, data }` variant + match arms in `From` impls | +14 |
| `parser/frontmatter.rs` | Rule frontmatter fields (category, rationale, example, anti_pattern) | +10 |
| `parser/mod.rs` | Rule type parsing + frontmatter serialization | +15 |
| `graph.rs` | `"rules"` dir → `PageType::Rule` inference | +1 |
| `WIKI-MEM.md` | Add `## Rules` section | +5 |
| **Total** | | **~72** |
