---
title: Fix sync_skills_to() recursive + platform mapping in setup
type: task
status: done
tags: [from-spec, go-mode]
priority: high
knowns_id: n7oz3d
spec: specs/wm-sdd-skills
fulfills: [AC-4, AC-5, AC-6, AC-7, AC-8, AC-11]
relates_to:
  - {type: implements, target: wiki:specs:wm-sdd-skills}
---

# Fix sync_skills_to() recursive + platform mapping in setup

> **Spec:** `specs/wm-sdd-skills`

> **Fulfills:** AC-4, AC-5, AC-6, AC-7, AC-8, AC-11

> *Imported from Knowns task `n7oz3d`*

# Fix sync_skills_to() recursive + platform mapping in setup

## Description


Fix `sync_skills_to()` in main.rs to handle subdirectory structure recursively (not flat file copy). Add platform→skill-dir mapping matching Knowns: `.claude/skills/` for claude-code, `.kiro/skills/` for kiro, `.agents/skills/` for all others (opencode, codex, cursor, gemini, antigravity, agents). Add `wm setup all` to sync to all three dirs. Wire `wm setup <platform>` to sync embedded skills via rust-embed.


## Acceptance Criteria
