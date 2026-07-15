---
title: Skill system structure — 14 wm-* skills + flow orchestrator
type: memory
tags: [skills, workflow, mcp, sdd]
created_at: "2026-07-07T10:34:50.654Z"
updated_at: "2026-07-07T10:34:50.654Z"
---

Embedded skills live in wm-core/src/skills/wm-*/SKILL.md, compiled via rust-embed. 14 skills: init, research, spec, plan, implement, review, commit, verify, doc, extract, debug, template, go (pipeline), flow (orchestrator with parallel gate). Skills are synced to .agents/skills/ and registered as wm_skill.* MCP tools. Each skill has: frontmatter (name, description), announce line, preflight, step-by-step with MCP tool calls, checklist, red flags, next step suggestion. Skills reference wm_* MCP tools only.