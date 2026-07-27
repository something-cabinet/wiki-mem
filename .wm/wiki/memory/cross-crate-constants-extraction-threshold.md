---
title: Cross-crate constants extraction threshold
type: memory
tags: [pattern, constants, architecture]
status: active
---

Magic values used in 3+ crates → `wm-constants` shared package (zero deps). Used in 1-2 crates → per-crate `constants/` dir with barrel `mod.rs`. Full pattern: @wiki/patterns/cross-crate-constants