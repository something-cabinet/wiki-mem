---
title: WM-004 — Arbitrary file read and cross-origin exfiltration via wm_source
type: task
status: done
acceptance_criteria:
  - text: "add_source returns Err for paths outside a configured source_dirs entry (e.g. /etc/hosts and .git/config), including dot-files under an allowed root"
  - text: "No new config field is added — the existing source_dirs + source_extensions config is reused in strict mode, and discover_sources inherits the validation without duplication"
  - text: "Grandfathered sources still process via stored_path, rejections emit tracing::warn!, and cargo clippy/check --workspace emit zero warnings"
---


Severity: Critical

`add_source` reads any path with no root confinement, and `wm_source process` returns the bytes in the HTTP response. With `Access-Control-Allow-Origin: *` a hostile page reads the response cross-origin. Verified live against `/etc/hosts` and an arbitrary secrets file.

Escalated to Critical because `.git/config` is readable this way and holds a GitHub PAT, and the CI publishes to npm on `v*` tag push — so drive-by read leads to supply-chain compromise. Root-confinement alone does NOT fix this: `.git/` is inside the project root. Strict mode (dot-component rejection) is required.

Per D2 the fix reuses the existing `source_dirs` + `source_extensions` config rather than adding a new field.

## Acceptance Criteria

- [ ] RED: `add` with `/etc/hosts` returns `Err`, failing before the fix
- [ ] RED: `add` with `.git/config` returns `Err` — name the test after the PAT exposure path
- [ ] GREEN: `add_source` accepts a path only if it confines under a configured `source_dirs` entry AND matches `source_extensions`, using strict mode
- [ ] A `.md` file under a configured `source_dirs` entry still ingests, and `process` returns its content
- [ ] A dot-file under an allowed root is still rejected
- [ ] Grandfathered sources still process — `process` and `verify` read `stored_path`, so no migration is needed
- [ ] `discover_sources` inherits validation with no duplicated checks
- [ ] No new config field; nothing added to `ProjectConfig`
- [ ] Rejections emit `tracing::warn!`
- [ ] `cargo clippy --workspace` and `cargo check --workspace` emit zero warnings

## Files

- `apps/wm-core/src/source_service.rs` (:13-25 `add_source`, unconfined read at :24; `discover_sources` at :279-333 already enforces the allowlist)
- `apps/wm-core/src/mcp/tools/source.rs` (:52-59 Add/Process arms)
- `apps/wm-core/src/config/models/project_config_model.rs` (`source_dirs`, `source_extensions` already present)

## Notes

`add_source:24` is the sole arbitrary-read primitive. `claim_source_and_read_content:122` and `verify_source:214` both read `stored_path` inside `.wm/sources/`; `original_path` is only written to the registry at :49 and compared as a dedup key. Fixing this one call closes the finding entirely.

## Implementation Notes (2026-08-08)

- Confinement was already implemented (`source_dirs` + `source_extensions` + `confine_strict` in `source_service.rs:56-64,28-37`); this lane added the RED tests that were missing.
- Tests (RED before fix) in `apps/wm-core/tests/security_test.rs`:
  - `wm004_etc_hosts_is_rejected` — `/etc/hosts` returns `Err` with `Access denied`.
  - `wm004_git_config_pat_exposure_is_rejected` — `.git/config` (GitHub PAT exposure path) rejected.
  - `wm004_dotfile_under_allowed_root_is_rejected` — a dot-file under an allowed `source_dirs` entry is rejected by strict mode.
  - `wm004_allowed_source_still_ingests` — a `.md` under `docs/` still ingests (`state: pending`).
- All pass; no config field added, `discover_sources` inherits the validation unchanged.
