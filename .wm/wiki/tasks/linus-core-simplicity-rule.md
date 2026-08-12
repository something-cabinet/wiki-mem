---
title: 'Task: add Linus-core simplicity rule to wiki rules'
type: task
id: wiki:tasks:linus-core-simplicity-rule
status: done
acceptance_criteria:
- "Spec page exists (@wiki/specs/linus-core-simplicity-rule) with functional + non-functional requirements"
- "Rule page .wm/wiki/rules/no-compensating-layers.md exists: type rule, status active, id wiki:rules:no-compensating-layers"
- "Rule covers the 5 core principles (compensating layers / no self-HTTP / honesty / test quality / dead things die) with rg enforcement probes"
- "Related section cross-links ≥4 existing rules + critical-patterns"
- "wm lint check and wm validate pass; rule page appears in the rules set"
implementation_notes: "2026-08-12: rule page authored (5 requirements + rg probes + exceptions + related). Probes verified runnable. wm lint PASS (635 nodes), wm validate PASS. Spec + task + rule all committed-ready (uncommitted; user decides)."
---

## Finding

2026-08-12 architecture + test reviews ("layers of stupidity, each compensating the other"): ~5,300 lines of compensating machinery (CLI-over-HTTP, mcp-token theater, 401 retry loop, spawn/probe/port midwifery, SSE stub justification, daemon-spawning tests, env-gated silent-green tests, kill -9 teardown) were deleted. The principles that made the simple architecture obvious deserve a binding rule so the pattern cannot quietly return.

## Files

- .wm/wiki/rules/no-compensating-layers.md (new rule)
- .wm/wiki/specs/linus-core-simplicity-rule.md (this spec)
- Related rules gain a Related back-reference (optional, non-blocking)

## Implementation notes (2026-08-12)

Rule authored (5 requirements + rg probes + exceptions + related), probes verified runnable. Rule reviewed and approved by the user (active). wm lint PASS (635 nodes, 680 edges), wm validate PASS. Uncommitted — user decides on commit.

