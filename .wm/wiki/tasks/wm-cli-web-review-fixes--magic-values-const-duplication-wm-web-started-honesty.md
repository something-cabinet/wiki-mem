---
title: wm-cli web review fixes — magic values, const duplication, wm-web started honesty
type: task
tags:
- bug
- cli
- wm-server
- refactor
- review-findings
status: done
priority: high
acceptance_criteria:
- text: No magic buffer size [0u8; 4096] in http_status or tests — named const
  checked: false
- text: 127.0.0.1 literal replaced with a named const (shared or local)
  checked: false
- text: Read-timeout Duration::from_secs(1) replaced with named const
  checked: false
- text: READY_DEADLINE_SECS no longer duplicated with divergent values — single shared source (wm-constants)
  checked: false
- text: wm-web started only logged when GET / returns 2xx; timeout/non-2xx logs a clear note instead of claiming started
  checked: false
- text: cargo check --workspace + clippy clean, tests pass
  checked: false
relates_to:
  - {type: implements, target: wiki:specs:wm-cli-web-review-fixes}
---

Fix 4 rule violations + 1 logic finding from review of commit 93449f9 (wm-cli web lifecycle + --port): V1 magic buffer 4096; V2 repeated 127.0.0.1 literal; V3 magic 1s read timeout; V4 READY_DEADLINE_SECS duplicated with divergent values (prod 10 / test 30); L1 'wm-web started' logged without confirmation on timeout/non-2xx. Do NOT push — leave in working tree so v0.3.7 release CI can finish.