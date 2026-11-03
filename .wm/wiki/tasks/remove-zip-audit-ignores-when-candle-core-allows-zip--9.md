---
title: Remove zip audit ignores when candle-core allows zip >= 9
type: task
id: "wiki:tasks:remove-zip-audit-ignores-when-candle-core-allows-zip--9"
status: todo
priority: medium
tags: [security, dependencies, audit, follow-up]
acceptance_criteria:
  - checked: false
    text: The four RUSTSEC ids (0367, 0368, 0369, 0429) are no longer in .cargo/audit.toml
  - checked: false
    text: Cargo.lock resolves zip >= 9.0.0
  - checked: false
    text: cargo audit passes without the ignores
---

`.cargo/audit.toml` ignores RUSTSEC-2026-0367/0368/0369/0429 because candle-core 0.11.0 pins zip ^8.6.0 and never calls the vulnerable extract APIs. Remove those ignores once a candle-core release allows zip >= 9.0.0 (or the decision feature is dropped), then verify `cargo audit` stays clean.