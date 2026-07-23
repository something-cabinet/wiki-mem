---
title: relates_to slash→colon normalization fix
type: concept
tags: [graph, relates_to, id, bug]
---

relates_to targets in frontmatter use inconsistent ID schemes — half use `/` separators (`wiki:specs/graph-and-ui-fix`) while node IDs use `:` separators (`wiki:specs:graph-and-ui-fix`). The `id_index` lookup silently dropped ~53% of entries (~26 edges).

**Fix:** Added `target.replace('/', ':')` normalization before lookup at `apps/wm-core/src/graph/mod.rs:130`. Also added `tracing::debug!` log for still-unresolved targets.

**Commit:** `fb28a96`