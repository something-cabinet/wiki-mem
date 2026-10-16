---
title: Before deleting a shim, grep the path form not just the names
type: memory
tags: [refactor, grep, verification]
status: active
---

A one-line re-export was called harmless by an inventory and a review alike, and had a live consumer using `crate::vector_db::SectionDoc`. A re-export is invisible to a name grep. Grep both the exported names and the path the shim provides, repoint the path consumers, then delete. Full reference: @wiki/patterns/alias-shim-consolidation