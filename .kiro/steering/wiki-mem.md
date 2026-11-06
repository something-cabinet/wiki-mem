---
description: Wiki Memory Engine guidelines — always included so the agent follows repo conventions.
---

# Wiki Memory Engine Guidelines

This steering file ensures the agent reads the canonical project guidance on every interaction.

## Priority

Loading the rule pages and core docs is the first action of every session. It comes before answering, planning or any other tool call, and it is never skipped. Activate the `wm-init` skill first. Do not read `.wm/wiki` with shell commands while the wm tools are available. See "Priority: load rules and core docs first" in WIKI-MEM.md, included below.

#[[file:WIKI-MEM.md]]
