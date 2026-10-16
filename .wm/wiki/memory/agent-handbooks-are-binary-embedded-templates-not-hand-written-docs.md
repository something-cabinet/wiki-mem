---
title: Agent handbooks are binary-embedded templates, not hand-written docs
type: memory
tags: [documentation, agent-guidance, templates]
status: active
---

AGENTS.md/CLAUDE.md/WIKI-MEM.md and platform configs are generated from templates under apps/wm-core/src/embed_files/shims/ and re-copied by `wm-cli init`/`agents`. Fix the template first, then sync the checked-in copies and assert byte-equality. Fixing only the copies helps this repo and leaves every user's project wrong. Full reference: @wiki/patterns/fix-templates-not-generated-copies