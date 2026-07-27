---
title: Curated clippy lint list + clippy.toml
type: memory
tags: [decision, clippy, lint]
status: active
---

Clippy uses curated lints in Cargo.toml `[workspace.lints.clippy]` + `clippy.toml` for config. No `all = warn`. Restriction-group lints (`as_conversions`, `cast_*`) cause worse code. Named `#[allow]` with reason permitted for correct code. Decision: @wiki/decisions/clippy-lint-curated-list-not-all