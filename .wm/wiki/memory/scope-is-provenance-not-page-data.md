---
title: Scope is provenance, not page data
type: memory
tags: [architecture, identity, memory]
status: active
---

Encoding location as an enum variant on one page type's data model (MemoryLayer::Global) was the original category error: it was never persisted or read. Location belongs to the composer, not the page. Full reference: @wiki/decisions/scope-is-provenance-not-page-data