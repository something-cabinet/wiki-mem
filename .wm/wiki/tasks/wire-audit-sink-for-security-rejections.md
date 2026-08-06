---
title: Wire the dead audit sink for security rejections
type: task
status: todo
acceptance_criteria:
  - text: "The audit channel drained at main.rs:27-30 is wired to a real sink instead of discarding, and security rejections (path escape, disallowed tool, auth failure) emit audit events"
  - text: "Events persist to .wm/log.jsonl and are queryable via wm_log, with attacker-controlled strings escaped or truncated before persistence"
  - text: "Repeated rejections are visible as a pattern; cargo clippy and cargo check emit zero warnings"
---

Severity: Medium

Spillover from the security remediation spec (D3). Path-confinement rejections log via `tracing::warn!` only, which is ephemeral and easy to miss in MCP stdio mode. A prompt-injected agent probing for traversal should leave a durable, queryable trace.

An audit channel already exists but is a no-op sink: `apps/wm-server/src/main.rs:27-30` receives from `audit_rx` and discards every message.

## Acceptance Criteria

- [ ] The audit channel drained at `main.rs:27-30` is wired to a real sink instead of discarding
- [ ] Security rejections (path escape, disallowed tool, auth failure) emit audit events
- [ ] Events persist to `.wm/log.jsonl` and are queryable via `wm_log`
- [ ] Attacker-controlled strings are escaped or truncated before persistence
- [ ] Repeated rejections are visible as a pattern, not just individual lines
- [ ] `cargo clippy --workspace` and `cargo check --workspace` emit zero warnings

## Files

- `apps/wm-server/src/main.rs` (:27-30 dead audit sink)
- `apps/wm-core/src/mcp/tools/log.rs`
- `.wm/log.jsonl`

## Notes

Injection hazard: `.wm/log.jsonl` sits inside the tree that agents read. Writing raw attacker-controlled paths into it would turn the audit log into a prompt-injection vector. Escaping is a requirement, not a nicety.
