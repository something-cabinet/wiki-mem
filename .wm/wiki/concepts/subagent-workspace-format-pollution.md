---
title: 'Failure: Subagent workspace-wide cargo fmt pollutes working tree'
type: concept
id: wiki:concepts:subagent-workspace-format-pollution
status: draft
tags: [failure, subagents, orchestration, cargo-fmt, workflow]
relates_to:
  - {type: references, target: wiki:tasks:apply-oracle-recommendations-from-linus-critique-review}
---

## What went wrong

A parallel fixer subagent ran `cargo fmt` or `cargo clippy --fix` workspace-wide during a focused remediation campaign. This touched 37 files outside its assigned lane, adding unrelated formatting churn to the working tree. The orchestrator had to restore all non-lane files to HEAD before committing.

## Root cause

Subagent prompts instructed "verify with cargo clippy -D warnings" but did not explicitly forbid workspace-wide formatting tools. The subagent interpreted a clippy finding as actionable and ran an auto-fix that reformatted everything.

## Prevention

In subagent prompts for focused lanes:
1. State explicitly: "Do NOT run cargo fmt, cargo fix, or cargo clippy --fix on the workspace."
2. Limit verification commands to the specific crate/package: `-p wm-core`, not the workspace root.
3. After subagent work lands, always `git diff --stat HEAD` to detect unexpected file-count inflation before committing.
4. If detected: `git checkout HEAD -- <non-lane files>` to restore, then re-verify green.

## Time lost

~5 minutes (detection + selective restore + re-verify). Low cost because it was caught before commit, but would have been a noisy PR diff if not caught.

## Related

- @wiki/tasks/apply-oracle-recommendations-from-linus-critique-review
- @wiki/memory/parallel-fixer-agents-summary