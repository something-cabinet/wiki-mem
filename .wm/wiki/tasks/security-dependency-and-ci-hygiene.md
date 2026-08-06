---
title: Dependency and CI hygiene from the security review
type: task
status: todo
acceptance_criteria:
  - text: "fast-uri bumped out of 3.0.0-3.1.4 (GHSA-7p8r-x3mc-p8w7) and postcss bumped past 8.5.22 (GHSA-fxqj-rqcc-2cmp)"
  - text: "serde_yaml 0.9.34+deprecated replaced (RUSTSEC-2024-0320) with cargo test --workspace green, and orphaned spartan-ng-brain-1.1.0.tgz deleted"
  - text: "cargo audit and npm audit --omit=dev installed, run, triaged, and added as CI gates; permissions: contents: read added to ci.yml; CSP added to index.html; .env gitignored; empty server_discovery.rs removed"
---

Severity: Low

Spillover from the security remediation review. Non-security hygiene, deliberately kept out of the remediation spec so the security diff stays small and reviewable.

## Acceptance Criteria

- [ ] `fast-uri` bumped out of 3.0.0-3.1.4 (GHSA-7p8r-x3mc-p8w7, high, production dependency)
- [ ] `postcss` bumped past 8.5.22 (GHSA-fxqj-rqcc-2cmp, moderate)
- [ ] `serde_yaml 0.9.34+deprecated` replaced (RUSTSEC-2024-0320, unmaintained)
- [ ] `spartan-ng-brain-1.1.0.tgz` deleted — orphaned, referenced by no lockfile, and the lock resolves 1.1.1 from the registry
- [ ] `cargo audit` installed, run, and added as a CI gate; findings triaged
- [ ] `npm audit --omit=dev` added as a CI gate
- [ ] `permissions: contents: read` added at workflow top level — `ci.yml` currently has no permissions block anywhere
- [ ] Content-Security-Policy added to `index.html`, with a nonce or hash for the existing inline theme script
- [ ] `.env` added to `.gitignore`
- [ ] `apps/wm-server/src/server_discovery.rs` removed — the file is empty but declared at `main.rs:7`
- [ ] `cargo test --workspace` green after the `serde_yaml` migration

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
