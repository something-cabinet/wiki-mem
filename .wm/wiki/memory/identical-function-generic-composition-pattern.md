---
title: Identical-function → generic composition pattern
type: memory
tags: [pattern, refactoring, boilerplate, rust]
status: active
---

When 3+ functions share identical structure with only data varying, extract a private generic fn. Each variant becomes a thin data-only wrapper. Saves ~120 lines from symbols_helper. Full reference: @wiki/patterns/identical-function-composition