---
id: 5r0d3a
title: 'Add rust-embed dep + create 13 wm-* skill files'
status: done
priority: high
labels:
  - from-spec
  - go-mode
createdAt: '2026-07-07T03:39:47.752Z'
updatedAt: '2026-07-07T03:46:18.569Z'
timeSpent: 0
spec: specs/wm-sdd-skills
fulfills:
  - AC-3
---
# Add rust-embed dep + create 13 wm-* skill files

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
(1) Add `rust-embed` to wm-core/Cargo.toml dependencies. (2) Create `wm-core/src/skills/` directory with 13 subdirectory files (wm-init, wm-research, wm-plan, wm-spec, wm-implement, wm-review, wm-commit, wm-verify, wm-doc, wm-extract, wm-debug, wm-go, wm-template), each containing `SKILL.md` with WM-native `wm_*` tool references. (3) Add `#[derive(RustEmbed)] struct SkillAssets` in skill.rs pointing to `src/skills/`.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

