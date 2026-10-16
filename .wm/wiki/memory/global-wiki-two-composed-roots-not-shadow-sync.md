---
title: Global wiki layer removed - manual duplication accepted (reversed 2026-10-15)
type: memory
tags: [architecture, global, design]
status: active
---

REVERSED 2026-10-15: the global wiki layer was removed and manual per-project duplication accepted. Reason: the global rule set is a statement of the owner's preferences across projects, so deciding which rules go global and which variant is canonical is the owner's call - the curation pass showed the mechanism pushes that judgement somewhere it does not belong. The measured need is real (46 rule pages, four names for one rule, same-name content differing 4x) and is now accepted as duplication rather than solved by composition. The two-root design, scope-as-provenance-as-mechanism, shadow reporting and promote/demote are all withdrawn. What survives: wiki:specs/configurable-wm-root (relocating one project's state root) and wiki:specs/remove-global-memory-layer. See the Superseded note on wiki:specs/global-wiki-layer.