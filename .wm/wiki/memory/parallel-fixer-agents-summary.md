---
title: "Parallel Fixer Agents for Batch File Editing"
type: memory
tags: [workflow, delegation, batch]
created_at: "2026-07-24"
relates_to:
  - {type: references, target: wiki:patterns:parallel-fixer-agents}
---

Dispatch parallel fixer subagents each handling 2–8 files for batch operations. Categorize by module, give explicit per-type rules, run straggler pass after. Used for ~90-file comment removal in 7 agents. Full reference: @wiki/patterns/parallel-fixer-agents
