---
title: 'Pattern: Fix the template, not the generated copy'
type: pattern
id: "wiki:patterns:fix-templates-not-generated-copies"
status: draft
tags: [pattern, documentation, agent-guidance, templates, workflow]
relates_to:
  - {type: relates_to, target: wiki:concepts:failure-guard-covered-only-one-representation}
---

## Problem

Agent handbooks and platform configs that a tool generates for its users are usually shipped as templates embedded in the binary. The checked-in copies in the repo look like the source of truth, but they are generated output. Fixing the copies and not the embedded template fixes this repo and does nothing for users — and the next `init`/`agents` run re-copies the stale text and silently overwrites any hand edit.

## Solution

Treat the embedded template as the source of truth and the checked-in files as build artifacts:

1. Fix the template under the embed directory (e.g. `apps/wm-core/src/embed_files/shims/`) first.
2. Re-sync the checked-in copies from the template, and make that a **verified equality**, not a hand edit (`diff -q` each pair, or an assertion in the test suite).
3. Add a guard that fails when the two diverge.
4. When a spec deletes a user-facing surface (a command, a flag), the embedded templates are part of that surface. The removal is not complete until the templates are updated.

A doc-update acceptance criterion with no automated check is the one that gets skipped. In the case this was learned from, the code half of a removal landed, the doc half did not, and the templates kept teaching deleted commands for months — to every new project, not just this one.

## When to Use

- Any repo that generates agent-facing docs, MCP configs, or scaffolding from templates compiled into the binary.
- When reviewing a spec that removes or renames a CLI command or flag — ask where the generated docs live.

## When Not to Use

- Hand-maintained docs with no generator: there the file really is the source of truth, and syncing logic would be overhead.

## Related

- @wiki/tasks/agent-handbooks-reference-nonexistent-cli-commands — the case this came from
- @wiki/specs/remove-self-install-flow — the completed spec whose doc-update AC was skipped
- @wiki/concepts/failure-guard-covered-only-one-representation — the guard written for this, and the hole it had