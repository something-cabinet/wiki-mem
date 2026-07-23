---
title: Embed shim files as compile-time assets
type: spec
status: approved
tags: [refactor, shims, rust-embed, generator, implemented]
---

## Overview

Shim files (AGENTS.md, CLAUDE.md, GEMINI.md, OPENCODE.md, REASONIX.md, .github/copilot-instructions.md) are compatibility entrypoints that delegate to `WIKI-MEM.md`. Currently they're generated at runtime from hardcoded `format!()` strings in `sync_agent_files()`. This is fragile — any change to the shim template requires modifying Rust source code instead of editing a template file.

The project already has a proven pattern for this: skills are embedded via `rust_embed::RustEmbed` from `apps/wm-core/src/skills/`. Shim templates should follow the same approach.

## Functional Requirements

- [x] FR-1: All shim templates exist as standalone `.md` files under a dedicated directory
- [x] FR-2: Templates are embedded at compile time via `#[derive(RustEmbed)]`
- [x] FR-3: `sync_agent_files()` copies templates from embedded assets to target paths
- [x] FR-4: Title line is customized per shim (e.g., `# AGENTS`, `# CLAUDE`) — via per-file embedded content
- [x] FR-5: No `format!()` string templates remain in `main.rs` for shim content

## Non-Functional Requirements

- NFR-1: Must match the existing `SkillAssets` embedding pattern
- NFR-2: Generator output must be byte-identical to current output (or semantically identical for WIKI-MEM.md redirects)

## Approach

1. Create `apps/wm-core/src/shim_templates/` directory with one `.md` file per shim
2. Add a RustEmbed struct
3. Update `sync_agent_files()` to read from embedded assets

## Locked Decisions

- D-1: Templates live in `apps/wm-core/src/shim_templates/` (consistent with `apps/wm-core/src/skills/`)
- D-2: No per-platform specialization — all shims follow WIKI-MEM.md delegate pattern
