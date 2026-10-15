---
title: Capture Rust learning research (borrowing semantics + inspection tooling) as durable wiki knowledge
type: task
id: "wiki:tasks:capture-rust-learning-research-borrowing-semantics--inspection-tooling-as-durable-wiki-knowledge"
status: in-review
priority: medium
tags: [docs, rust, learning, knowledge-capture]
spec: specs/rust-learning-knowledge-capture
acceptance_criteria:
  - text: "howto/rust-inspect-variables-and-layout exists as a valid record-bearing page and covers: {:p} / &raw const / addr_of!, size_of / align_of / offset_of! / Layout, -Zprint-type-sizes, cargo expand / cargo asm, tracing + lldb/CodeLLDB + rust-analyzer, Miri, ASan/dhat"
  - text: "howto records the wiki-mem-specific caveats: dbg! is effectively banned (dbg_macro = warn + no-warnings), no-comments-in-code forbids explanatory probes, Miri cannot run the ort/tree-sitter/tokio FFI surface, LeakSanitizer has no aarch64-apple-darwin target, -Zprint-type-sizes needs CARGO_INCREMENTAL=0"
  - text: "patterns/borrow-errors-order-area-layout documents order / area / layout with the verified anchors: engine_state_mediator.rs:22-27, page_crud_service.rs:110, page_update_builder_service.rs:15, parser_helper.rs:13-18, onnx/mod.rs:101-104"
  - text: "concepts/rust-learning-research-sources registers the course, episode, exercise repo and the per-episode coverage, and states that the video series continues with reborrowing/lifetimes"
  - text: "Every new page passes wm_validate.check for its entity id (canonical questions, no prose body), and wm_page.get returns a record warning-free body"
relates_to:
  - {type: references, target: wiki:specs:rust-learning-knowledge-capture}
  - {type: references, target: wiki:howto:rust-inspect-variables-and-layout}
  - {type: references, target: wiki:patterns:borrow-errors-order-area-layout}
  - {type: references, target: wiki:concepts:rust-learning-research-sources}
---

**Finding:** Two research passes produced knowledge that currently exists only in chat and will be lost:
1. Borrowing semantics — RustCurious course episode 10 ("Three Ways to Fix Any Borrowing Error"), transcript-verified, mapped to verified `file:line` ownership sites in this workspace.
2. Runtime inspection tooling — how to view addresses, layout, and variable state in Rust, with the wiki-mem-specific constraints that invalidate the generic advice.

**Why it matters:** The three-approach framework (order / area / layout) is the missing mental model behind several wiki-mem design decisions (`ArcSwap` co-swap, `.rcu()` copy-on-write, per-path write locks). The inspection ladder is what makes ownership and refcount behaviour *observable* rather than theoretical.

**Sources (verified):**
- `https://www.youtube.com/watch?v=dLx8usb759E` — "10. Three Ways to Fix Any Borrowing Error", RustCurious, 38:55, 2026-09-20
- Course playlist `PLfeE8CbRe3DWFvoDEXYt25dd8cjfD0nzf`; exercises `https://rustcurious.com/10`
- std/Rust Reference/cargo/Miri/CodeLLDB/rust-analyzer official docs (see spec)

**Note:** `findings-first-task-spec` requires a task + spec page. One task + one spec cover all three deliverable pages because they are a single finding ("capture this research") delivered as three artefacts — not three separate findings.