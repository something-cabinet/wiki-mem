---
title: "Decision: Barrel Files Required"
type: decision
status: approved
tags: [decision, barrel, module-structure]
decision:
  context: "Direct file-path imports create fragile references. Moving a file breaks every import. The codebase already uses barrel files in many modules (page/mod.rs, search/mod.rs) but not consistently."
  options:
    - "Direct imports from individual files"
    - "Barrel files (mod.rs) re-exporting everything"
  rationale: "Barrel files decouple consumers from file layout. You can rename, split, or merge files inside a module directory without touching any consumer. This is the standard Rust pattern."
  outcome: "Every module directory MUST have a mod.rs Barrel that re-exports all public items. No consumer imports from individual files."
---
