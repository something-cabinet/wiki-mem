---
title: wm health audit CLI command
type: memory
tags: [cli, health, audit]
status: active
---

`wm health audit` is a CLI command that scans wiki health. Default mode is dry-run. Use `--fix` to apply fixes. Flags: `--dry-run`, `--fix`, `--format json|text`. Detects empty pages (no parseable sections), broken relates_to refs (target pages that don't exist), and graph cycles. Fix mode: deletes stale empty task pages (0 inbound refs), case-corrects broken refs when the target exists with different casing, and removes truly broken refs.