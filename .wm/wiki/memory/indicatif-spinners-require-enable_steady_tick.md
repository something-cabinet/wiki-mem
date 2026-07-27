---
title: indicatif spinners require enable_steady_tick
type: memory
tags: [cli, rust, indicatif, failure]
status: active
---

indicatif spinner won't animate without enable_steady_tick(duration). Always call it after new_spinner() to start the background ticker. Full docs: @wiki/howto/indicatif-cli-progress, failure: @wiki/concepts/spinner-without-steady-tick