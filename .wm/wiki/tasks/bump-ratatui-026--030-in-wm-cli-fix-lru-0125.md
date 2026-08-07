---
title: Bump ratatui 0.26 → 0.30 in wm-cli (fix lru 0.12.5)
type: task
id: wiki:tasks:bump-ratatui-026--030-in-wm-cli-fix-lru-0125
status: done
priority: low
tags:
- security
- deps
- rust
- tui
acceptance_criteria:
- text: ratatui upgraded to ^0.30 in apps/wm-cli/Cargo.toml
- text: TUI code compiles with ratatui 0.30 API (cargo build -p wm-cli)
- text: cargo tree -i lru shows no lru 0.12.x in tree
- text: wm-cli TUI renders and exits cleanly (smoke test)
implementation_notes: 'Done 2026-08-06: ratatui 0.26 -> 0.30.2. Changes: Cargo.toml ratatui 0.30 (dropped direct crossterm 0.28 dep -> ratatui::crossterm re-export 0.29); tui.rs f.size() -> f.area() x2; crossterm imports via ratatui::crossterm. cargo build -p wm-cli passes. lru bumped 0.12.5 -> 0.18 via ratatui-core (clears GHSA-rhfx-m35p-ff5j / RUSTSEC-2026-0002).'
---

lru 0.12.5 (GHSA-rhfx-m35p-ff5j, Stacked Borrows UB in IterMut) comes transitively via ratatui 0.26.3 in wm-cli. ratatui 0.30.x dropped the lru dependency entirely, so bumping removes the vulnerable dep. Major bump requires API migration (Terminal init, Frame/Backend trait changes in 0.27+).

From dependabot sweep 2026-08-06: only remaining runtime Rust alert; low severity (soundness), defer-safe but tracked.