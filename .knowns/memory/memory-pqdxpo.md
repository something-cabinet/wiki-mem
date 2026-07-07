---
id: pqdxpo
title: Knowns/WM is a memory layer, not a spec system
layer: project
category: decision
tags:
  - memory
  - knowns
  - architecture
  - openspec
createdAt: '2026-07-07T03:52:29.276Z'
updatedAt: '2026-07-07T03:52:29.276Z'
---

OpenSpec (@fission-ai/openspec) is a dedicated spec system with change folders, lifecycle, and cross-repo Stores. Knowns/WM specs are a thin technique (Socratic exploration + doc template) on top of the memory engine. Knowns' real value is the memory substrate: typed graph edges, semantic retrieval, context assembly, cross-references, AC tracking. WM should double down on the memory layer, not try to compete with OpenSpec. Full reference: @doc/learnings/learning-knowns-memory-layer-not-a-spec-system
