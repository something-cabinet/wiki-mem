---
title: "Pattern: wm init --full — Chain System + Project Setup"
type: pattern
status: reviewed
tags: [pattern, setup, bootstrap, knowns]
relates_to:
  - {type: references, target: wiki:decisions:wm-self-upgrade}
  - {type: references, target: wiki:specs:wm-self-install}
---

## Problem

New developers or fresh clones need multiple steps to start working: install the binary, configure MCP, initialize the project. Each step requires a different command with different scopes (system vs project).

## Solution

A single `wm init --full` command that chains three operations:
1. `wm upgrade` — copies binary to `~\.wm\bin\`, registers on PATH
2. `wm setup opencode` — writes MCP config using PATH-based `wm-cli` command
3. `wm init` — creates `.wm/wiki/` project structure

## When to Use

- Setting up WM on a new machine
- Bootstrapping a fresh clone of a WM project
- First-time user onboarding

## Related

- @wiki/decisions/wm-self-upgrade
- @wiki/specs/wm-self-install
