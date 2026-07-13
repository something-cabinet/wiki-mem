---
title: The Fantasy Benchmark — Compete Against Expectations, Not Reality
description: The first Knowns audit was wrong about Knowns but right about what users expect. The fantasy version of a competitor is a better design target than the real one.
createdAt: '2026-07-10T08:57:26.455Z'
updatedAt: '2026-07-10T08:57:26.455Z'
tags:
  - learning
  - strategy
  - product
  - benchmarking
  - critical
---

# Learning: The Fantasy Benchmark

## The Discovery

During a competitive audit of Knowns (Go project memory system), the first analysis by an AI agent made **multiple incorrect assumptions** about Knowns' capabilities:

| Assumed (Fantasy) | Reality (Actual Knowns) |
|---|---|
| Session memory layer | Only project + global. Session returns error. |
| Skill execution engine | Skills are just returned as text. No execution. |
| Tree-sitter code intelligence | Tree-sitter was **removed**. LSP-only now. |
| Rich inline @template resolution | Only reference parsing, not content resolution. |
| TUI | **No TUI at all.** |

When a second audit read the **actual Knowns source code**, most of these assumptions were wrong.

**But the fantasy version of Knowns was more compelling than the real one.**

## The Pattern: Compete Against Expectations

### What happened
The first oracle described what an agent *expected* a project memory tool to have — session memory, skill execution, tree-sitter, reference resolution. These weren't accurate descriptions of Knowns. They were **user expectations looking for a product**.

### Why this matters
- Users compare your product against their **mental model**, not your competitor's source code
- Knowns succeeds despite NOT having session memory, skill execution, or a TUI
- If WM ships these features, it doesn't match Knowns — it **leapfrogs** by becoming what people assumed Knowns should have been
- The "fantasy" is a **design target**, not a mistake

### When to apply
- When benchmarking against a competitor, always read their actual code — but also ask: "what would a user assume this product does?"
- The gap between assumptions and reality is the **opportunity space**
- If your competitor doesn't ship what people assume they have, you can

### What to avoid
- Mistaking the fantasy for reality in audit conclusions (first audit error)
- Ignoring the fantasy entirely (you miss the design target)
- Building only to parity against real competitor (you match mediocrity)

## The Meta-Lesson

This session produced a meta-lesson about **how to use AI auditors**:

1. **First-pass audit**: Will produce an idealized comparison (fantasy) because the agent fills gaps with assumptions
2. **Second-pass audit**: With actual competitor code, reveals reality
3. **Synthesis**: The fantasy is the better design target; the reality is the honest delta

This pattern applies whenever an AI agent is asked to compare two systems without reading both codebases. The output will naturally trend toward the idealized version of the less-examined system. That output is not **truth** — it's **aspiration**. Use it as such.

## Strategic Implication for WM

WM should **not** replicate Knowns' actual feature set. It should ship:

1. **Session memory** — Knowns doesn't have it; users expect it
2. **Skill execution** (event-driven) — Knowns doesn't have it; WM has the trigger infrastructure
3. **Tree-sitter code intelligence** — Knowns removed theirs; WM could have the only one
4. **@reference auto-resolution** inline in bodies — Knowns partially has it; WM should fully ship it
5. **Web UI** — Knowns has one; WM needs one for parity
6. **Template engine** — Knowns has a real one; WM needs to catch up

Items 1-3 are where WM leapfrogs. Items 4-6 are where WM matches what Knowns already shipped.
