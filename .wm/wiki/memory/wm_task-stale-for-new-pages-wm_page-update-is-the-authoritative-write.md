---
title: wm_task stale for new pages — wm_page.update is the authoritative write
type: memory
status: active
tags: [tool-reliability, mcp, graph-index, fixed]
---

Phantom "page not found" on wm_page.update / wm_task.update for pages that exist on disk = stale in-memory graph index (write paths had no disk fallback while get did). FIXED 2026-08-07 via shared graph-first/disk-fallback resolver (resolve_page_meta in page_crud_service.rs); no workaround needed anymore. Full reference: @wiki/patterns/stale-index-disk-fallback