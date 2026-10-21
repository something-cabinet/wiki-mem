---
title: Webpki advisory bump — rustls-webpki and root stores
type: spec
id: "wiki:specs:webpki-advisory-bump"
status: approved
tags: [security, dependencies, audit, spec]
---

schema_version: 1
state: |-
  ## Goal

  Clear the three webpki advisories that failed Cargo audit in CI run 39633586288 by taking the patched releases, not
  by adding ignores.

  ## Verified current state

  - `rustls-webpki 0.103.15` — RUSTSEC-2026-0382, patched >= 0.103.16.
  - `webpki-roots 1.0.8` — RUSTSEC-2026-0381, patched >= 1.0.10; the 0.26.11 compatibility crate is unaffected.
  - `webpki-root-certs 1.0.8` — RUSTSEC-2026-0383, patched >= 1.0.10.
  - All three are transitive TLS dependencies; no workspace source references them directly.

  ## Change

  `cargo update -p rustls-webpki@0.103.15 -p webpki-roots@1.0.8 -p webpki-root-certs@1.0.8`
  -> 0.103.16 / 1.0.10 / 1.0.10. No source change, no new ignores.

  ## Verification

  - `cargo audit` exits 0.
  - `cargo check -p wm-cli -p wm-core` passes.
  - CI check job green.

  ## References

  - @wiki/tasks/bump-rustls-webpki-and-webpki-roots-for-rustsec-2026-038103820383
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
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers:
  kind: system
  scope: local
  status_class: approved
  needs_tasks: true
