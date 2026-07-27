---
title: Review wm init — opencode.json not generated during init
type: task
id: wiki:tasks:review-wm-init--opencodejson-not-generated-during-init
status: todo
priority: medium
tags: [init, setup, opencode, mcp]
---

wm init does not generate opencode.json (the MCP config file for OpenCode). opencode.json is only generated via `wm setup opencode`. This means a fresh `wm init` produces a project that isn't MCP-connected until the user separately runs `wm setup`. Review the init flow and decide whether opencode.json (and equivalent MCP configs for other platforms) should be generated as part of init, or if the separate `wm setup` step is intentional. If the gap is real, add MCP config generation to the init wizard step.