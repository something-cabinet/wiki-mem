---
title: Don't conflate Knowns' features — code-gen vs platform config are separate
type: memory
tags: [failure, research, knowns]
status: active
---

When researching Knowns patterns, don't conflate their code generation system (knowns template run, uses Handlebars) with their platform config generation (built with Go map literals). These are different features with different needs. Verify which feature you're comparing. Reference: @wiki/concepts/handlebars-hbs-rabbit-hole