---
title: Mutation-test a guard across every representation it must catch
type: memory
tags: [testing, guard, mutation-testing]
status: active
---

A guard that only parsed fenced ```bash blocks passed while an inline `wm-cli bogus` sailed through — and the real defect contained both forms. Before trusting a guard, inject a violation in every representation (fenced block, inline span, prose, config snippet) and confirm each one fails. A guard that has never failed has not been tested. Full reference: @wiki/concepts/failure-guard-covered-only-one-representation