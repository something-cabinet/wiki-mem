---
title: dead_code ban — dead input fields masked by allow
type: memory
tags: [clippy, lint, dead-code, contract]
status: active
---

allow(dead_code) on API input fields hides dead contract fields (issue #126 root cause: wm_doc.r#type declared but never wired). Banned repo-wide 2026-08-14, CI grep enforces; use expect(dead_code) which errors when the lint stops firing (self-cleaning). No clippy lint can ban attributes. Full reference: @wiki/decisions/clippy-lint-curated-list-not-all