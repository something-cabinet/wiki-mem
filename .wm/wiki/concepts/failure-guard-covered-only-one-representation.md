---
title: 'Failure: Guard test covered only one representation of the thing it guarded'
type: concept
id: "wiki:concepts:failure-guard-covered-only-one-representation"
status: draft
tags: [failure, testing, guard, mutation-testing]
relates_to:
  - {type: references, target: wiki:tasks:shim-template-command-parity-guard}
---

## What went wrong

A guard test was written to fail when an agent-handbook template names a CLI command that does not exist. It passed. It also passed after a deliberately invalid command was injected — as long as the injection was placed **inline** (`\`wm-cli bogus\``) rather than inside a fenced ```bash block. The guard only parsed fenced code blocks.

## Root cause

The test matched one *representation* of the thing it guarded (a fenced shell block) and was assumed to cover the concept (any documented invocation). The defect it was written for contained **both** representations — a fenced block *and* an inline sentence like `Run \`wm mcp\` to start the MCP server` — so the guard would have missed half the regression class it existed for.

## Prevention

- Mutation-test the guard: inject a violation in *every* representation the guarded thing can take, and confirm each injection fails. A guard that has never failed has not been tested.
- Enumerate representations explicitly before writing the matcher — here: fenced bash blocks, inline code spans, prose command mentions, config and JSON snippets, table cells.
- Keep the "did it match anything" assertion (`checked > 0`). A matcher that silently matches nothing is indistinguishable from a clean pass.
- Make the failure message name the file and the line; a guard that fails without naming the site gets switched off.

## Time lost

About 20 minutes. The guard was written, reported green, and re-verified as green before an independent mutation pass found the hole.

## Related

- @wiki/tasks/shim-template-command-parity-guard — the guard, its hole, and the required verification on re-add
- @wiki/specs/shim-template-command-parity-guard — amended to require inline-span coverage
- @wiki/patterns/fix-templates-not-generated-copies — the change the guard exists to protect