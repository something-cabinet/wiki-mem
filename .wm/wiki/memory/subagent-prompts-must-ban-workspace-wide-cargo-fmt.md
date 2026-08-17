---
title: Subagent prompts must ban workspace-wide cargo fmt
type: memory
tags: [workflow, subagents, cargo-fmt]
status: active
---

Subagent prompts for focused lanes must explicitly ban workspace-wide cargo fmt/fix. Always git diff --stat HEAD after subagent work lands to catch unexpected file-count inflation. Restore non-lane files to HEAD before committing. Full reference: @wiki/concepts/subagent-workspace-format-pollution