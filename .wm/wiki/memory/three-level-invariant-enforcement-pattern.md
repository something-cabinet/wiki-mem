---
title: Three-level invariant enforcement pattern
type: memory
tags: [pattern, invariant]
status: active
---

When enforcing a project invariant, use three levels: (1) Architecture — make it impossible via types/fn signatures (e.g., MainEngine::new() detects root internally). (2) Runtime lint — wm_lint.check scans for violations (e.g., rogue .wm/ walkdir scan). (3) CI — fail the build. Each level catches what the previous misses.