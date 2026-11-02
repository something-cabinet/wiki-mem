---
title: Clear the new cargo audit advisories
type: spec
id: "wiki:specs:clear-the-new-cargo-audit-advisories"
status: draft
tags: [security, dependencies, ci, spec]
acceptance_criteria:
  - text: The zstd, zstd-safe and zstd-sys advisories are fixed by a lockfile bump
  - text: cargo audit exits 0 on master
  - text: The CI check job is green on master and on the v0.6.1 tag, so publish can start
---

schema_version: 1
state: |-
  ## Goal

  The CI check job passes again, because publish and publish-npm both wait for it.

  ## Verified current state

  - The runs for the pushed v0.6.1 tag and for master failed in the check job. Two causes were mine and are fixed in e576be1: a wm-cli file over 300 lines, wm-core over its 13000-line ceiling, and a dev-dependency the layer gate denies in wm-engine.
  - The audit step is behind them and also fails. New advisories were published against crates in the lockfile; none comes from the commits of this release.
  - cargo audit reports 5 vulnerabilities: zstd 0.13.3 (RUSTSEC-2026-0391), zstd-safe 7.2.4 (0403 and 0398), zstd-sys 2.0.16 (0410) and zip 8.6.0 (0429).
  - The zstd family has compatible fixes: zstd 0.13.4, zstd-safe 7.4.0, zstd-sys 2.1.2. cargo update on those three packages moved the lockfile only; the workspace tests (832) and the CI library suites pass on it.
  - zip 8.6.0 is pinned by candle-core 0.11.0 (^8.6.0) and the fix needs zip 9. RUSTSEC-2026-0429 is ZipArchive::extract aborting on a symlink entry with a large declared size. candle-core has no call to extract (the only matches for extract in its source are SIMD lane extracts). The decision feature that pulls candle-core is off by default. Three earlier zip advisories are already ignored in .cargo/audit.toml for the same reason, with the removal task wiki:tasks:remove-zip-audit-ignores-when-candle-core-allows-zip--9.

  ## Options for zip

  - A. Add RUSTSEC-2026-0429 to the ignore list in .cargo/audit.toml, the way the three earlier zip advisories are handled, and extend the removal task to cover it. Recommended: it matches the existing decision and the vulnerable function is not called.
  - B. Wait for a candle-core that allows zip 9. Blocks the release for an unknown time.
  - C. Remove candle-core from the lockfile by dropping the optional decision feature. Large change for an off-by-default feature.

  ## Change

  - Commit the zstd lockfile bump.
  - Apply the owner's choice for zip.

  ## Verification

  - cargo audit exits 0.
  - The CI check job passes on the pushed commit.
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
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers:
  kind: system
  scope: project-wide
  needs_tasks: true