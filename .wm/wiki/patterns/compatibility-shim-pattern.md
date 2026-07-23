---
title: Compatibility shim pattern
type: pattern
status: draft
relates_to:
  - {type: references, target: wiki:tasks:task-wm-reasonix-integration}
---

## Problem

Multiple AI runtimes (Claude Code, OpenCode, Gemini, Copilot, Reasonix) each auto-detect a different filename at session boot (CLAUDE.md, OPENCODE.md, GEMINI.md, REASONIX.md). Each runtime expects to find its own instruction file, but maintaining separate full instruction sets across all these files creates drift and duplication.

## Solution

Use a thin compatibility shim pattern: each runtime-specific file is a minimal entrypoint that redirects to a single canonical source (WIKI-MEM.md). The shim contains:
- Auto-detection header (`# <NAME>` + "Compatibility entrypoint for runtimes...")
- `<!-- WIKI-MEM GUIDELINES START/END -->` markers for tooling detection
- A CRITICAL directive pointing to WIKI-MEM.md as canonical
- Minimal type-specific content (e.g., specialist skill list for Reasonix)
- Quick reference with essential commands

All shims follow the exact same template, differing only in the filename and header. This keeps maintenance low and ensures every runtime reaches the same canonical guidance.

## When to Use

- Adding support for a new AI runtime that auto-detects a specific filename
- Any project that wants runtime-agnostic canonical guidance
- When maintaining multiple instruction files becomes a burden

## When Not to Use

- If the runtime does not auto-detect a specific filename
- If the runtime requires runtime-specific instructions that don't apply to other runtimes

## Related
- @wiki/tasks/task-wm-reasonix-integration