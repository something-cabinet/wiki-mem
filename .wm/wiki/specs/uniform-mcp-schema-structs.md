---
title: "Uniform Schema Structs for All MCP Tool Actions"
page_type: spec
status: draft
tags: [spec, refactor, mcp, schema, uniformity]
relates_to:
  - {type: references, target: wiki:reference/design-patterns}
---

## Locked Decisions

- **D1 — Uniform Schema Structs**: Every MCP tool action variant with parameters gets a dedicated schema struct. No exceptions. No `#[allow(dead_code)]` on action enums.
- **D2 — Newtype wrapper**: Schema structs are `NewtypeSchema` (tuple struct wrapping the params) rather than named field structs. This keeps the handler match expression concise: `WmPageAction::List(s)` vs destructuring inline.
- **D3 — Naming**: `Wm{Domain}{Variant}Schema` — e.g., `WmPageListSchema`, `WmTaskCreateSchema`.

## Overview

Every MCP tool action with parameters gets a dedicated schema struct. Currently some action enums mix inline variant fields with `#[allow(dead_code)]` for fields that exist only for JSON Schema generation. This creates two patterns: inline fields for used params, `#[allow(dead_code)]` for schema-only fields.

The uniform pattern eliminates the judgment call entirely: **every variant with parameters → schema struct.**

## Motivation

- **Uniformity** — one pattern, zero exceptions. New dev knows: "add variant → create schema struct."
- **Discoverability** — `WmPageListSchema` can be grepped, documented, and inspected independently
- **Zero `#[allow(dead_code)]`** — no suppression needed anywhere
- **Correctness** — the schema struct IS the contract. What you see is what the MCP client sends.

## Action Enums Affected

### toolkit/action.rs (3 enums)

| Enum | Variants | Current state |
|---|---|---|
| `WmPageAction` | List, Get, Create, Update, Delete, Link, Unlink — 7 variants | `#[serde(tag = "action")]`. List has `limit` (schema-only). |
| `WmTaskAction` | Board, List, Create, Get, Update, Delete, CheckAc, UncheckAc, Subtask — 9 variants | List has `status, priority, assignee` (schema-only). |
| `WmTemplateAction` | List, Get, Create, Run — 4 variants | All fields used. |

### Flat tool files (10 files)

| File | Input type | Fields |
|---|---|---|
| `graph.rs` | `enum WmGraphAction` | Neighbors { id, depth, edge_type } — depth/edge_type schema-only |
| `time.rs` | `enum WmTimeAction` | Stop { note }, Add { note }, Report { group_by } — all schema-only |
| `memory.rs` | `enum WmMemoryAction` | Add { category } — schema-only |
| `index.rs` | `enum WmIndexAction` | Embed { force } — schema-only |
| `doc.rs` | `enum WmDocAction` | List { r#type } — schema-only |
| `code.rs`, `search.rs`, `validate.rs`, `project.rs`, `source.rs`, `reference.rs`, `skills.rs`, `log.rs`, `lint.rs`, `model.rs`, `version.rs`, `decision.rs` | Various input structs | — |

## Pattern

### Current (mixed)
```rust
#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
enum WmPageAction {
    List {
        r#type: Option<String>,
        #[allow(dead_code)]
        limit: Option<usize>,
    },
    Create { path: String, title: String, content: Option<String> },
}
```

### Proposed (uniform)
```rust
#[derive(Deserialize, JsonSchema)]
struct WmPageListSchema {
    #[schemars(description = "Filter by page type")]
    pub r#type: Option<String>,
    #[schemars(description = "Maximum number of results")]
    pub limit: Option<usize>,
}

enum WmPageAction {
    List(WmPageListSchema),
    Create(WmPageCreateSchema),
}

// Handler
match action {
    WmPageAction::List(s) => { s.r#type; s.limit; }
    WmPageAction::Create(s) => { s.path; s.title; }
}
```

## Where schema structs live

Each action enum's schema structs go alongside the action enum, following the existing file structure:

```
mcp/tools/page/
  action.rs            ← WmPageAction enum + WmPage*Schema structs
  output.rs            ← output types
```

For flat files, schema structs go in the same file as the action enum.

## Schema struct naming

`Wm{Domain}{Variant}Schema` — e.g., `WmPageListSchema`, `WmTaskCreateSchema`, `WmGraphNeighborsSchema`.

Exception: single-variant input structs (not action enums) keep their existing names.

## Acceptance Criteria

- [ ] AC-1: Every action variant with parameters has a schema struct
- [ ] AC-2: Zero `#[allow(dead_code)]` remains in action enums
- [ ] AC-3: All handlers compile and pass tests
- [ ] AC-4: `cargo check` has zero dead_code warnings
- [ ] AC-5: MCP JSON schema output is identical (verify with existing tool listings)
- [ ] AC-6: Handler code uses `..` for unused schema fields

## Execution Plan

### Phase 1: Flat files (quickest, ~10 files)
Each file with `#[allow(dead_code)]` on an enum → extract schema structs.
Files: `time.rs`, `memory.rs`, `index.rs`, `doc.rs`, `graph.rs`

### Phase 2: Action enum files (3 files)
`page/action.rs`, `task/action.rs`, `template/action.rs` — extract schema structs for all variants with parameters.

### Phase 3: Remove all `#[allow(dead_code)]`
After all schema structs are in place, remove every `#[allow(dead_code)]` annotation across all action enums.

## Non-Goals

- Not changing the tool registration mechanism
- Not changing handler dispatch logic
- Not changing MCP tool output types
