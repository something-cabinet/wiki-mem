---
title: ArcSwap copy-on-write for incremental index updates
type: memory
tags: [rust, graph, architecture]
status: active
---

For in-memory indices using ArcSwap, use copy-on-write for single-element mutations: load Arc, clone inner data, mutate clone, store new Arc. No reader blocking. Full reference: @wiki/patterns/arcswap-copy-on-write-incremental