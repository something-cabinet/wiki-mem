---
title: Wire the dead audit sink for security rejections
type: task
status: done
acceptance_criteria:
  - text: "The audit channel drained at main.rs:27-30 is wired to a real sink instead of discarding, and security rejections (path escape, disallowed tool, auth failure) emit audit events"
  - text: "Events persist to .wm/log.jsonl and are queryable via wm_log, with attacker-controlled strings escaped or truncated before persistence"
  - text: "Repeated rejections are visible as a pattern; cargo clippy and cargo check emit zero warnings"
---

Severity: Medium

Spillover from the security remediation spec (D3). Path-confinement rejections log via `tracing::warn!` only, which is ephemeral and easy to miss in MCP stdio mode. A prompt-injected agent probing for traversal should leave a durable, queryable trace.

An audit channel already exists but is a no-op sink: `apps/wm-server/src/main.rs:27-30` receives from `audit_rx` and discards every message.

## Acceptance Criteria

- [x] The audit channel drained at `main.rs:27-30` is wired to a real sink instead of discarding
- [x] Security rejections (path escape, disallowed tool, auth failure) emit audit events
- [x] Events persist to `.wm/log.jsonl` and are queryable via `wm_log`
- [x] Attacker-controlled strings are escaped or truncated before persistence
- [x] Repeated rejections are visible as a pattern, not just individual lines
- [x] `cargo clippy --workspace` and `cargo check --workspace` emit zero warnings

## Files

- `apps/wm-server/src/main.rs` (:27-30 dead audit sink)
- `apps/wm-core/src/mcp/tools/log.rs`
- `.wm/log.jsonl`

## Notes

Injection hazard: `.wm/log.jsonl` sits inside the tree that agents read. Writing raw attacker-controlled paths into it would turn the audit log into a prompt-injection vector. Escaping is a requirement, not a nicety.

## Implementation Notes (2026-08-08)

**Design — shared sink in wm-core (transport-independent):**

- New `apps/wm-core/src/shared/audit_sink.rs`:
  - `SecurityAuditEvent` (JSON line): `timestamp`, `category: "security"`, `kind` (`path_escape`|`hidden_path`|`disallowed_tool`|`invalid_model`|`source_rejected`|`auth_failure`), `tool`, `detail`, `path`.
  - `sanitize()` strips control characters and truncates attacker strings at 256 chars — a control-char newline cannot split the JSON line (tested).
  - `write_security_audit(project_root, &event)` appends to `<project_root>/.wm/log.jsonl`, creating `.wm/` if needed, never propagating errors. If no `.wm` dir exists under the derived root, the event is dropped (keeps library unit-test CWDs clean).
  - `derive_project_root(confine_root)` walks up to the `.wm` component; falls back to CWD — matching where `wm_log.*` reads.
- **Emission points (all in wm-core, so both the daemon and proxy paths inherit them):**
  - `path_confine_helper::confine` / `confine_strict` emit on every rejection (covers wm_doc, wm_page, wm_template, source_service).
  - `ToolRegistry::dispatch`/`dispatch_async` emit `disallowed_tool` on permission-check rejection.
  - `wm_model` emits `invalid_model` on bad/unknown names.
  - **Auth failures NOT wired**: the web-token and MCP-token middleware live in `apps/wm-server/src/routes/` — out of the allowed scope for this lane (scope: wm-core/src + wm-core/tests + .gitignore + index.html). `KIND_AUTH_FAILURE` is defined in the sink and the middleware can call it in a follow-up.
  - **The wm-server/wm-cli discard loops were NOT removed** (also out of scope). They are now harmless: security events bypass the in-memory channel entirely and are written to `.wm/log.jsonl` at the chokepoint, so the daemon path (wm-server → `dispatch_async` → tool → confine) and proxy path (wm-cli → HTTP → wm-server → …) both persist them. A follow-up can delete the loops and route the channel to the sink.
- `wm_log.*` now read `.wm/log.jsonl` via `engine.project_root` (not CWD) so the audit file is queryable from anywhere.
- Tests in `apps/wm-core/tests/security_test.rs`: `audit_sink_records_path_escape_rejection` (rejected `wm_doc` create leaves a `path_escape` line, re-queryable via `wm_log.filter`) and `audit_sink_sanitizes_control_characters` (injected `\u{0007}`/`\u{000a}` cannot split the line). Pass.
- Remaining for done: remove the two discard loops in `wm-server/src/main.rs` / wire auth-failure middleware emission.

## Implementation Notes (2026-08-08) — follow-up lane, DONE

**Audit channel wired (main.rs):** the drain loop at `main.rs:44-47` no longer discards. `wm-server` now drains `audit_rx` into the shared sink on a spawned task, persisting each `AuditEvent` (from `EngineState::emit_audit`) as a `category:"tool"`/`kind:"tool_call"` JSON line via the new `write_tool_audit` helper. Attacker-controlled fields are sanitized/truncated like the security events; serde_json escaping keeps one event on one physical line.

**Auth failures wired (routes/mod.rs + routes/mcp.rs):** `KIND_AUTH_FAILURE` is now emitted from both rejection branches:
- `require_token` (web API layer) — `POST /api/code/symbols` without a token now writes `{"category":"security","kind":"auth_failure","tool":"http_auth","detail":"POST /api/code/symbols"}` to `<root>/.wm/log.jsonl`.
- `require_mcp_token` (privileged MCP channel) — same emit for `/api/mcp/*` rejections.
- The detail is `{method} {path}` only — the supplied credential is never logged. Both middlewares capture `project_root` from `engine.project_root` at router-build time (never CWD).
- New helper `audit_sink::audit_auth_failure(project_root, detail)` keeps the event construction in wm-core.

**Proxy path coverage (wm-cli):** `apps/wm-cli/src/mcp_proxy.rs` has NO audit channel of its own and no discard loop — it forwards `tools/list`/`tools/call` over HTTP to the daemon's `/api/mcp/*` channel, where `require_mcp_token` (auth) and the wm-core chokepoints (confine/disallowed/invalid-model) emit to the sink. So the proxy path is fully covered daemon-side; nothing to wire in wm-cli.

**Test:** `wm_cli_web_auth_failure_leaves_audit_line` (in `apps/wm-core/tests/wm_cli_web_test.rs`) spawns the real daemon, POSTs `/api/code/symbols` without a token, asserts 401 and that `root/.wm/log.jsonl` contains `"kind":"auth_failure"`, `"category":"security"`, and the rejected route. Passes end-to-end (12/12 in `wm_cli_web_test`, 8/8 `e2e_http`, 20/20 `security_test`; `cargo clippy --workspace -- -D warnings` and `cargo check` clean).
