---
title: Rust Learning Research Sources
type: concept
id: "wiki:concepts:rust-learning-research-sources"
status: draft
tags: [rust, learning, research, sources]
---

schema_version: 1
state: |-
  ## Purpose

  Provenance register for the Rust learning material captured in `@wiki/howto/rust-inspect-variables-and-layout` and `@wiki/patterns/borrow-errors-order-area-layout`, so the sources survive and the material can be re-checked or extended later.

  ## Primary source

  - **Course**: RustCurious, "Complete Rust course" playlist - `https://www.youtube.com/playlist?list=PLfeE8CbRe3DWFvoDEXYt25dd8cjfD0nzf`
  - **Episode**: 10, "Three Ways to Fix Any Borrowing Error" - `https://www.youtube.com/watch?v=dLx8usb759E` (38:55, uploaded 2026-09-20)
  - **Presenter**: Ben, RustCurious (10 years teaching Rust)
  - **Exercises**: `https://rustcurious.com/10`
  - **Stable identifiers**, in case the URL dies: channel RustCurious, video id `dLx8usb759E`, playlist id `PLfeE8CbRe3DWFvoDEXYt25dd8cjfD0nzf`, episode 10.

  ## What episode 10 covers

  | Timestamp | Segment |
  |---|---|
  | 0:00 | Intro - a refactor produced cannot borrow `*self` as mutable more than once |
  | 1:47 | Visualising ownership - Mutable XOR Shared as a compile-time reader-writer rule |
  | 11:37 | Reborrowing - explicit and implicit, and shared reborrows of mutable references |
  | 21:59 | Partial borrows - field-level tracking, and why a `&mut self` method widens the borrow |
  | 31:21 | Practice exercise - `ChannelMonitor` with `Option<Sender>` and two counters, fixed three ways |

  ## Where the series goes next

  Episode 10 closes by announcing that reborrowing is revisited through *invisible lifetimes*, followed by *named lifetimes*. Any use of struct fields that hold references should wait for that material; episode 10 deliberately stops short of lifetime syntax.

  ## Secondary sources

  Official Rust documentation used for the inspection material: Rust Reference (type layout), std docs (`core::ptr`, `core::fmt::Pointer`, `std::mem::offset_of!`, `core::alloc::Layout`, `std::macro::dbg!`), Rust Performance Book (type sizes), Unstable Book (`print-type-sizes`, sanitizer), Miri README, CodeLLDB manual, rust-analyzer Book, and the cargo book (`cargo tree`).

  ## Honesty notes

  - Episode content is **transcript-derived**, so quotations are faithful to the captions rather than re-checked against the audio.
  - Tooling stability claims (stable versus nightly, and the release an item stabilised in) come from the official documentation listed above.
  - Code snippets marked as verified in the derived pages were produced by a research subagent on its own machine and were **not** all re-run during this capture. Only the repository `file:line` anchors and the lint and toolchain facts were re-verified here.
  - The authoring machine reported `1.96.0-nightly` as the default toolchain, so stable release numbers cited may differ from a reader toolchain.

  ## Constraints

  - No wiki `rule` page is derived from this material yet. If the order/area/layout framework proves load-bearing, promote it through the `wm-extract` flow.
  - No exercise code is vendored into this repository.

  ## Related

  - `@wiki/howto/rust-inspect-variables-and-layout`
  - `@wiki/patterns/borrow-errors-order-area-layout`
questions:
  - id: kind
    type: choice
    instructions: What kind of concept document is this?
    options:
    - concept
    - failure-analysis
    - research-report
    - reference-note
  - id: category
    type: choice
    instructions: Which domain category does this concept belong to?
    options:
    - architecture
    - search-retrieval
    - graph
    - parser-format
    - mcp-tooling
    - cli
    - storage
    - embeddings
    - web-ui
    - process
  - id: maturity
    type: score
    instructions: How mature is the understanding of this concept?
    levels:
    - raw
    - exploratory
    - established
    - stable
  - id: code_referenced
    type: noul
    instructions: This concept references concrete code.
answers:
  kind: research-report
  category: process
  maturity: exploratory
  code_referenced: true
