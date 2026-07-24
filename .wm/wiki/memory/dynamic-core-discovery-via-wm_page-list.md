---
title: Dynamic core discovery via wm_page.list
type: memory
tags: [decision, init, core]
status: active
---

wm-init now discovers core pages dynamically via wm_page.list({'type': 'core'}) instead of hardcoded IDs. README loaded explicitly. Full reference: @wiki/decisions/dynamic-core-discovery-over-hardcoded-ids