---
title: WM adopts Knowns SDD workflow (kn-* skills, not gh-*)
type: memory
tags: [workflow, skills, knowns, sdd]
created_at: "2026-07-07T03:18:15.650Z"
updated_at: "2026-07-07T03:18:15.650Z"
---

Decision: WM should follow the same SDD workflow as Knowns, using the kn-* skill set (init, research, plan, spec, implement, review, commit, verify, doc, extract, debug, go, template) instead of the custom gh-* skills (ingest, plan, implement, commit).

This requires:
1. **Replace `generate_default_skills()`** — output the 13 kn-* skills (subdirectory format) instead of 4 gh-* flat files
2. **Fix `scan_skills()`** — properly parse `kn-*/SKILL.md` subdirectory format; use filename stem from parent directory, not from SKILL.md
3. **Fix frontmatter parsing** — read `name:` field (Knowns convention) in addition to/superseding `title:`
4. **Adapt skill content** — kn-* skills must reference WM MCP tools (`wm_*` namespace) instead of Knowns tools (`mcp_knowns_*`)

Reference: Knowns v0.20.5 skill stubs synced to `.agents/skills/kn-*/SKILL.md` with full SDD workflow content.