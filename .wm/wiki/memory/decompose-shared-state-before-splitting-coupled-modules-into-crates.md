---
title: Decompose shared state before splitting coupled modules into crates
type: memory
tags: [architecture, domain-driven, refactor, crates]
status: active
---

Modules that all reach through one god-object cannot be split into crates with closed interfaces. Decompose the shared state into per-domain handles first (struct-of-handles facade, one domain per change); only then decide per domain whether a crate is earned. Splitting the application layer is the wrong axis - domains sit under it, not in it. Full reference: @wiki/patterns/decompose-shared-state-before-splitting-crates