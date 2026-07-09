---
id: 67fpsz
title: Skill directory convention — .agent/skills/ not .agents/
layer: project
category: convention
tags:
  - skills
  - convention
  - path
createdAt: '2026-07-09T08:18:32.462Z'
updatedAt: '2026-07-09T08:18:32.462Z'
---

WM skills should be synced to .agent/skills/ (not .agents/). Matches Knowns convention. Both `git add` and `wm setup agents` use .agent/skills/. Verified: wm-cli/src/main.rs has 6 references to .agent/skills/.
