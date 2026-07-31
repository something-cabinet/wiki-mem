---
title: CLI flags must be wired into behavior, not just acknowledged
type: memory
tags: [cli, ux, testing]
status: active
---

A clap flag that only logs "acknowledged" without threading the value into the underlying function is a silent no-op — users get a false sense of control (wm index code --skip-hash-check did nothing for 0.3.x). Wire the flag or remove it; add a test exercising the flag's path. Full ref: @wiki/concepts/inert-cli-flags-silent-noop