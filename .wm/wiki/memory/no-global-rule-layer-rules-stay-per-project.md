---
title: No global rule layer - rules stay per-project
type: memory
tags: [rules, architecture, decision]
status: active
---

Decided 2026-10-15: no global wiki layer. Rules stay per-project and the measured duplication (46 pages, four names for one rule, same-name content differing 4x) is accepted deliberately. A global rule set is a statement of the owner's preferences, so which rules go global is the owner's call - not a mechanism's. ARCHITECTURE's single-root invariant stands unmodified. configurable-wm-root survives; global memory removal survives. Full reference: @wiki/decisions/no-global-rule-layer-per-project-rules