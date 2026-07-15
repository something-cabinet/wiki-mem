---
title: Skill directory convention — .agent/skills/ not .agents/
type: memory
tags: [skills, convention, path]
created_at: "2026-07-09T08:18:32.462Z"
updated_at: "2026-07-09T08:18:32.462Z"
---

WM skills should be synced to .agent/skills/ (not .agents/). Matches Knowns convention. Both `git add` and `wm setup agents` use .agent/skills/. Verified: wm-cli/src/main.rs has 6 references to .agent/skills/.