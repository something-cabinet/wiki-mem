---
title: Cargo audit — scoped ignores for the candle-core zip advisories
type: spec
id: "wiki:specs:cargo-audit-zip-advisories"
status: approved
tags: [ci, security, audit, dependencies, spec]
---

schema_version: 1
state: |-
  ## Goal

  Restore a green `Cargo audit` step in CI without silently accepting an unreachable vulnerability: record why the
  three zip advisories do not apply to this workspace, and track the upgrade that removes the exception.

  ## Verified current state

  - CI run 39400658945 fails at `Cargo audit` with 3 vulnerabilities, all in `zip 8.6.0`.
  - Advisories (issued 2026-10-17, all fixed in zip >= 9.0.0):
    - RUSTSEC-2026-0367 — SUID/SGID bits preserved during ZIP extraction.
    - RUSTSEC-2026-0368 — `ZipArchive::extract` symlink target escapes the destination.
    - RUSTSEC-2026-0369 — permissions applied through symlinks without confinement.
  - Dependency path: `zip` <- `candle-core 0.11.0` (`zip = "8.6.0"`, caret) <- optional `decision` feature in
    `apps/wm-core` (off by default; see `apps/wm-core/Cargo.toml`).
  - candle-core 0.11.0 uses `zip::ZipArchive::new`, `by_name`, and `ZipWriter` only; it never calls
    `ZipArchive::extract` or `ZipStreamReader::extract` (verified by grep over the vendored source).
  - candle-core 0.11.0 is the latest published version, so `zip 9` cannot be selected by cargo. A semver patch
    cannot substitute 9.x for a `^8.6.0` requirement.

  ## Decision

  Add `.cargo/audit.toml` with `[advisories] ignore` for the three IDs and a written rationale. Do not fork or patch
  zip, do not lower the audit gate, and do not remove the decision feature as part of this fix.

  ## Risks

  | Risk | Consequence | Mitigation |
  |---|---|---|
  | Ignore outlives the reason | vulnerable extract API becomes reachable later | follow-up task; rationale names the exact unreachable APIs |
  | candle-core starts calling extract | advisory applies | dependency review when candle updates; remove ignores then |
  | A zip 8.x backport appears | stale ignore | re-check advisories when bumping |

  ## Verification

  - `cargo audit` reports `0 vulnerabilities` locally.
  - CI `check` job passes the `Cargo audit` step and completes green.
  - The ignore file records the three IDs, the dependent crate, and the follow-up task id.

  ## References

  - @wiki/tasks/ci-cargo-audit-fails-on-three-zip-advisories-pulled-by-candle-core
  - @wiki/tasks/remove-zip-audit-ignores-when-candle-core-allows-zip-9
questions:
  - id: kind
    type: choice
    instructions: What kind of spec is this?
    options:
    - feature
    - system
    - doc
    - migration
    - experiment
  - id: scope
    type: choice
    instructions: How wide is the scope of this spec?
    options:
    - local
    - component
    - system
    - project-wide
  - id: status_class
    type: choice
    instructions: What lifecycle class is this spec in?
    options:
    - draft
    - reviewed
    - approved
    - superseded
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers:
  kind: system
  scope: local
  status_class: approved
  needs_tasks: true
