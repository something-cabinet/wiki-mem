---
id: wiki:concepts:missed-project-guidance-fjadra
title: Session Init Failures — didn't read AGENTS.md, didn't research fjadra
type: concept
status: draft
tags: [failure, process, session-init, dependency]
relates_to:
  - {type: references, target: wiki:specs:graph-and-ui-fix}
  - {type: references, target: wiki:specs:fjadra-wasm-layout}
  - {type: references, target: wiki:patterns:critical-patterns}
  - {type: references, target: wiki:tasks:update-wm-init-load-rules}
  - {type: references, target: wiki:specs:wiki-rules-auto-load}
---
id: wiki:concepts:missed-project-guidance-fjadra

## Failure 1: Didn't read AGENTS.md / WIKI-MEM.md at session start

### What went wrong
During `wm-init` for this session, I skipped reading AGENTS.md, OPENCODE.md, and WIKI-MEM.md. These files contain critical project guidance:
- AGENTS.md → redirects to `.wm/AGENTS.md` for conventions + workflow
- OPENCODE.md → redirects to `WIKI-MEM.md` as canonical
- WIKI-MEM.md → enterprise correctness, tool rules, active rules, critical rules
- `.wm/wiki/rules/` → 3 active rules (no comments, use tuistory, tool reliability tracking)

WIKI-MEM.md (line 209) says: *"All rules under `@wiki/rules/` are binding — load and obey every active rule at session start."*
OPENCODE.md (line 7) says: *"CRITICAL: You MUST read and follow WIKI-MEM.md in the repository root before doing any work."*

Neither was followed.

### Root cause
The `wm-init` skill has the steps, but I didn't execute the full workflow. Specifically:
- Didn't read `.wm/AGENTS.md` 
- Didn't read `WIKI-MEM.md`
- Didn't load `.wm/wiki/rules/`
- Missed 3 active rules

### Prevention
- Follow wm-init completely — every step, every file.
- Load `.wm/wiki/rules/` at session start (3 active rules today).
- Read WIKI-MEM.md before any implementation.

### Time lost
~20 minutes of back-and-forth correcting missed guidance.

---
id: wiki:concepts:missed-project-guidance-fjadra

## Failure 2: Added fjadra as a server dependency without checking its purpose

### What went wrong
During the graph layout implementation (FR-2), I added `fjadra = "0.2"` to `wm-server/Cargo.toml` and wrote layout code using its API. fjadra is designed as a WASM-targeted library for browser-side force-directed layout — not a server-side Rust dependency. The user corrected this.

### Root cause
I assumed fjadra was a standard Rust library suitable for server use without checking:
- What deployment target it was designed for
- Whether it had wasm-bindgen in its dependency tree
- Whether d3-force (already available client-side) would suffice

### Prevention
- Before adding any new dependency, check its purpose, deployment target, and feature flags.
- Ask: is there already a simpler alternative (e.g., d3-force in JS)?
- For WASM-targeted crates, consider compiling to WASM for frontend use instead.

### Time lost
~15 minutes writing + reverting the fjadra dependency and implementation.

---
id: wiki:concepts:missed-project-guidance-fjadra

## Common thread
Both failures stem from the same habit: moving to implementation without reading the project's existing guidance first. The fix is the same: read before doing.

## References
- @wiki/patterns/critical-patterns
- @wiki/tasks/23b628
- @wiki/specs/wiki-rules-auto-load
