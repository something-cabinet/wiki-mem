---
title: Learning: Knowns = Memory Layer, Not a Spec System
type: decision
tags: [learning, architecture, memory, knowns]
relates_to:
  - {type: references, target: wiki:tasks:29fizw}
  - {type: references, target: wiki:concepts:specs:wm-sdd-skills}
---

## Pattern

### Spec-as-Technique vs Spec-as-System

- **What:** Knowns' `/kn-spec` is a Socratic exploration technique + doc template on top of its memory engine. OpenSpec (`@fission-ai/openspec`) is a dedicated spec system with change folders, lifecycle (propose→apply→archive), and cross-repo Stores.
- **When to use:** Use Knowns/WM specs for lightweight decision capture that links into the knowledge graph. Use OpenSpec when you need a full spec lifecycle with change artifact management.
- **Source:** @wiki/tasks/29fizw, @wiki/concepts/specs/wm-sdd-skills

## Decisions

### Knowns Core = Memory, Not Spec

- **Chose:** Knowns specs are a thin workflow layer (`/kn-spec` → Socratic Q&A → doc template). The core value is the **memory substrate**: typed graph edges, semantic search, cross-referenced docs/tasks/memories, AC tracking with coverage reports.
- **Over:** Treating specs as a standalone product feature that competes with OpenSpec.
- **Tag:** GOOD_CALL
- **Outcome:** WM's `wm-spec` skill mirrors Knowns' lightweight approach — just enough structure to capture requirements and link them into the graph. No attempt to build a change lifecycle system.
- **Recommendation:** OpenSpec is the right tool for spec lifecycle management. Knowns/WM is the right tool for persistent memory. They complement each other — use OpenSpec for the propose→apply→archive loop, use Knowns/WM for the knowledge graph that persists across sessions.

## Failures

None — this was a clarification, not a discovery from failure.