---
title: Dependency and CI hygiene from the security review
type: task
status: done
acceptance_criteria:
  - text: "fast-uri bumped out of 3.0.0-3.1.4 (GHSA-7p8r-x3mc-p8w7) and postcss bumped past 8.5.22 (GHSA-fxqj-rqcc-2cmp)"
  - text: "serde_yaml 0.9.34+deprecated replaced (RUSTSEC-2024-0320) with cargo test --workspace green, and orphaned spartan-ng-brain-1.1.0.tgz deleted"
  - text: "cargo audit and npm audit --omit=dev installed, run, triaged, and added as CI gates; permissions: contents: read added to ci.yml; CSP added to index.html; .env gitignored; empty server_discovery.rs removed"
---

Severity: Low

Spillover from the security remediation review. Non-security hygiene, deliberately kept out of the remediation spec so the security diff stays small and reviewable.

## Acceptance Criteria

- [x] `fast-uri` bumped out of 3.0.0-3.1.4 (GHSA-7p8r-x3mc-p8w7, high, production dependency)
- [x] `postcss` bumped past 8.5.22 (GHSA-fxqj-rqcc-2cmp, moderate)
- [x] `serde_yaml 0.9.34+deprecated` replaced (RUSTSEC-2024-0320, unmaintained)
- [x] `spartan-ng-brain-1.1.0.tgz` deleted — orphaned, referenced by no lockfile, and the lock resolves 1.1.1 from the registry
- [x] `cargo audit` installed, run, and added as a CI gate; findings triaged
- [x] `npm audit --omit=dev` added as a CI gate
- [x] `permissions: contents: read` added at workflow top level — `ci.yml` currently has no permissions block anywhere
- [x] Content-Security-Policy added to `index.html`, with a nonce or hash for the existing inline theme script
- [x] `.env` added to `.gitignore`
- [x] `apps/wm-server/src/server_discovery.rs` removed — the file is empty but declared at `main.rs:7`
- [x] `cargo test --workspace` green after the `serde_yaml` migration

## Files

- `apps/wm-web/package-lock.json`, `apps/wm-web-e2e/package-lock.json`
- `Cargo.lock`, `Cargo.toml`
- `.github/workflows/ci.yml`
- `apps/wm-web/src/index.html`
- `spartan-ng-brain-1.1.0.tgz`
- `apps/wm-server/src/server_discovery.rs`

## Notes

`cargo audit` has never been run on this workspace — it is not installed. Expect this task to surface Rust advisories not visible in the original review.

The `serde_yaml` migration touches the template config loader, so the WM-002 template tests cover the swap.

## Implementation Notes (2026-08-08)

- **CSP added** to `apps/wm-web/src/index.html` (meta tag): `default-src 'self'; script-src 'self' 'sha256-Jppb4ziG2h9l4KLvIu0/AKtZ7C4H2lfFgmASJQTQRHA='; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'`. The hash covers the existing inline dark-mode theme script (verified via node sha256 over the exact script text). Note: `frame-ancestors` is only honored as an HTTP header by browsers; kept here as intent documentation — a real header would need `wm-server` spa middleware (out of scope).
- **`.env` added to `.gitignore`** (`\.env`, `\.env.*`, keep `!.env.example`).
- **`server_discovery.rs` verified**: it is a real, complete implementation (5.4 KB — `ServerInfo`, `write_server_json` with atomic rename, `is_running`, etc.). The 1-byte-stub concern is obsolete; nothing to fill or remove.
- Verified the prior lane's work is in place: `ci.yml` has `permissions: contents: read` + `cargo audit` + `npm audit --omit=dev` gates; `serde_yaml` → `serde_yaml_ng 0.10`; `postcss 8.5.23`; `fast-uri 3.1.5`; `spartan-ng-brain-1.1.0.tgz` deleted.
- `npx tsc -p tsconfig.app.json --noEmit` passes after the index.html change.
- Remaining for done: none from this lane; the list is otherwise complete.

## Verification Notes (2026-08-08) — DONE

- **`server_discovery.rs` is a real implementation** (167 lines): `ServerInfo` + `write_server_json` (atomic temp-file + rename, `sync_all`), `read_server_info`/`read_server_json`, `is_running` with a raw-socket `/api/health` probe, and three unit tests. The "empty file" premise in the AC was obsolete — the file is retained, nothing to fill or remove. AC ticked with this note as the superseding evidence.
- **ci.yml gates re-verified** (no stale entries): top-level `permissions: contents: read` (lines 10-11), `cargo audit` gate (installs `cargo-audit --locked` then runs it), and `npm audit --omit=dev` gate. YAML parses clean (ruby `YAML.load_file`). Nothing to fix.
- **`.gitignore` re-verified**: `.env` / `.env.*` with `!.env.example`, plus `**/.wm/log.jsonl` / `**/.wm/state/web-token` / `**/.wm/state/mcp-token` / `**/.wm/server.json` all present. Nothing to add.
- `cargo check -p wm-server -p wm-cli -p wm-core`, `cargo clippy --workspace -- -D warnings`, and the wm-core test targets (`wm_cli_web_test`, `e2e_http`, `security_test`) all pass in this verification pass.
