---
status: done
implementation_notes: 'Implemented 2026-07-31 via wm-flow (fast). Lane A (fixer): removed Upgrade variant+arm, --full field+install step+force-opencode block, resolve_mcp_binary() → constant "wm-cli", deleted apps/wm-core/src/install/ + lib.rs wiring, WIKI-MEM.md quick-ref lines. Lane B (orchestrator): archived spec/decision/pattern, updated memories, added durable memory. Verified: cargo build/test/clippy zero warnings, CLI --help omits upgrade/--full, SDD pass. Note: wm_task.update transition validator reads stale state (todo→in-progress persists but done transition fails) — used wm_page.update for final status.'
---

Remove the binary self-install mechanism (~/.wm/bin + PATH registration) which is redundant with the npm/cargo distribution channels. See @wiki/specs/remove-self-install-flow for locked decisions D1-D4.