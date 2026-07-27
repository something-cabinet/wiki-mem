---
title: dialoguer for Rust CLI prompts
type: memory
tags: [cli, rust, ux, pattern]
status: active
---

Use dialoguer for styled CLI prompts (Confirm, Select, MultiSelect). Arrow keys + space bar instead of bare stdin. Always guard with is_terminal() check and --no-wizard flag. Full docs: @wiki/howto/dialoguer-cli-prompts