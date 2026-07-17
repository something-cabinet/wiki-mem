---
title: "Formalize Remaining Behavioral Design Patterns"
page_type: spec
status: draft
tags: [spec, refactor, architecture, patterns]
relates_to:
  - {type: references, target: wiki:reference/design-patterns}
---

## Overview

The codebase informally uses several behavioral patterns (Command, Observer, Visitor, Iterator, Decorator, Composite). This spec formalizes them — making the pattern explicit in the type system rather than implicit in the code structure.

## Locked Decisions

- **D1 — Command Object Pattern**: Action enums (WmPageAction, WmTaskAction, WmTemplateAction) each variant becomes a struct implementing a `ToolCommand` trait with `execute()` → move logic out of giant match arms into command objects. Enables undo/redo, queuing, and serialization.
- **D2 — Observer Trait**: Replace raw `tokio::sync::mpsc` usage with a formal `Subscriber<T>` trait. Audit events and skill events get typed subscriber lists.
- **D3 — Visitor Over Pattern Match**: Large `match input { ... }` blocks in MCP tools get a visitor trait. Adding a new action means implementing a method on the visitor, not adding a match arm.
- **D4 — Formal Iterator Implementations**: Graph traversal (node_indices, neighbors) and page listing expose `impl Iterator` instead of collecting into Vec.
- **D5 — Decorator Layer Trait**: Permission checking, audit logging, and error handling become composable `Layer` wrappers around tool handlers instead of inline code in `dispatch()`.
- **D6 — Composite Interface**: Tree structures (WikiPageContent + SectionDoc, TaskVersionHistory + TaskVersion) get a formal `TreeNode` trait with `children()` method.

## Requirements

### FR-1: Command Trait for Action Enums

```rust
pub trait ToolCommand {
    type Output;
    fn execute(self: Box<Self>, engine: &Arc<EngineState>) -> Result<Self::Output, ToolError>;
}
```

Each variant of WmPageAction, WmTaskAction, WmTemplateAction becomes a struct implementing this trait. The register function stores `Box<dyn ToolCommand>` instead of a match arm.

**Tradeoff:** Adds ~3 lines of boilerplate per variant but eliminates the giant match arm files (page/mod.rs is 418 lines, task/mod.rs is 640 lines). Enables undo history and command queuing.

### FR-2: Subscriber Trait

```rust
pub trait Subscriber<T: Clone> {
    fn notify(&self, event: &T);
}
```

Replace `tokio::sync::mpsc::Sender<AuditEvent>` with a `Publisher<AuditEvent>` that holds `Vec<Box<dyn Subscriber<AuditEvent>>>`. Same for skill events.

**Tradeoff:** ~50 lines of generic plumbing. Enables adding subscribers without changing publishers.

### FR-3: Visitor Trait for Action Dispatch

```rust
pub trait PageActionVisitor<T> {
    fn visit_list(&self, input: ListInput) -> T;
    fn visit_get(&self, input: GetInput) -> T;
    fn visit_create(&self, input: CreateInput) -> T;
    // ...
}
```

**Tradeoff:** Reduces match arm duplication but adds trait maintenance when new actions are added.

### FR-4: Iterator Implementations

```rust
impl Iterator for GraphNodeIterator {
    type Item = (NodeIndex, WikiPageMeta);
    fn next(&mut self) -> Option<Self::Item>;
}
```

**Tradeoff:** More boilerplate for iteration but enables lazy evaluation and adapter chaining.

### FR-5: Layer Trait for Handler Decorators

```rust
pub trait Layer {
    fn wrap(&self, handler: Box<dyn ToolHandler>) -> Box<dyn ToolHandler>;
}
```

Permission checks, audit logging, error boundaries become layers that wrap handlers.

**Tradeoff:** Adds indirection but makes the decorator chain explicit and testable.

### FR-6: TreeNode Trait

```rust
pub trait TreeNode {
    fn children(&self) -> Vec<&dyn TreeNode>;
}
```

**Tradeoff:** Small interface that enables uniform tree operations across WikiPageContent, TaskVersionHistory, etc.

## Acceptance Criteria

- [ ] AC-1: Command trait defined + at least one action enum converted (WmPageAction)
- [ ] AC-2: Pattern match arms replaced with command dispatch in converted tool
- [ ] AC-3: Subscriber trait + Publisher generic type defined
- [ ] AC-4: Audit events use Publisher<AuditEvent>
- [ ] AC-5: Visitor trait defined for at least one action enum
- [ ] AC-6: At least one Iterator implementation (graph nodes or pages)
- [ ] AC-7: Layer trait defined + permission check extracted as a Layer
- [ ] AC-8: TreeNode trait defined + WikiPageContent implements it
- [ ] AC-9: cargo build --all-features succeeds
- [ ] AC-10: cargo test passes same count

## Non-Goals

- Converting ALL action enums in one pass — start with WmPageAction
- Replacing ALL match arms — only extract where the visitor adds value
- Adding undo history or command queuing in this pass — the Command trait enables it but doesn't implement it yet

## Execution Order

1. Command trait + convert WmPageAction (highest value, 418-line file shrinks)
2. Layer trait for dispatch decorators (cleanest separation)
3. Iterator for graph traversal (used in most MCP tools)
4. Subscriber trait for audit events (decouples publishers from consumers)
5. TreeNode for WikiPageContent (used by search)
6. Visitor for action dispatch (lowest value, defer if time is short)
