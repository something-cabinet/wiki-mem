---
id: n7oz3d
title: Fix sync_skills_to() recursive + platform mapping in setup
status: done
priority: high
labels:
  - from-spec
  - go-mode
createdAt: '2026-07-07T03:39:57.010Z'
updatedAt: '2026-07-07T03:46:19.343Z'
timeSpent: 0
spec: specs/wm-sdd-skills
fulfills:
  - AC-4
  - AC-5
  - AC-6
  - AC-7
  - AC-8
  - AC-11
---
# Fix sync_skills_to() recursive + platform mapping in setup

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Fix `sync_skills_to()` in main.rs to handle subdirectory structure recursively (not flat file copy). Add platform→skill-dir mapping matching Knowns: `.claude/skills/` for claude-code, `.kiro/skills/` for kiro, `.agents/skills/` for all others (opencode, codex, cursor, gemini, antigravity, agents). Add `wm setup all` to sync to all three dirs. Wire `wm setup <platform>` to sync embedded skills via rust-embed.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

