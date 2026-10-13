---
title: All standard wiki pages are structured records
type: memory
tags: [structured, typed-decisions, wiki, migration]
status: active
---

All standard wiki page types are now structured records ({schema_version,state,questions,answers}); consumers read `state`; validator + migration planner share one directory-based record scope. 755/771 pages converted; the 16 prose residues are index.md (generated), 11 learnings + conventions + research (non-standard dirs, excluded), and 2 opted-out specs. Full: @doc/decisions/all-wiki-pages-structured-records