---
title: Materialize expensive passes at index time, not query time
type: memory
tags: [pattern, architecture, performance]
status: active
---

Run expensive global analysis (symbol resolution, type inference) once at index time and persist results. Query paths read pre-computed data from a `resolved_edges` table. Fallback to on-the-fly when table is empty. Full reference: @wiki/patterns/materialize-expensive-passes-at-index-time