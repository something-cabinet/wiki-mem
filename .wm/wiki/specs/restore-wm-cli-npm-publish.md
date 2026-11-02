---
title: Restore wm-cli npm publish in release CI
type: spec
id: "wiki:specs:restore-wm-cli-npm-publish"
status: approved
tags: [ci, release, npm, spec]
acceptance_criteria:
  - text: ci.yml has a v* tag gated publish-npm job that needs publish and publishes only wm-cli
  - text: After a tag push, npm view @something-cabinet/wm-cli version returns the tagged version
---

schema_version: 1
state: |-
  ## Goal

  Make a pushed `v*` tag publish `@something-cabinet/wm-cli` and its four platform packages to npm again, so `npm i -g @something-cabinet/wm-cli` delivers the released version.

  ## Verified current state

  - npm `latest` is 0.5.2 (2026-08-17); no 0.6.x exists on the registry.
  - Commit aa49715 (`chore(0.6): mcp-only`) deleted the whole `publish-npm` job while retiring wm-server and wm-web. The remaining `publish` job only builds the four targets and uploads workflow artifacts.
  - `gh release list` is empty, so there is no other distribution channel.
  - Tag runs 40270884474 and 40287785743 for v0.6.1 are fully green, and their binaries are discarded.
  - `apps/wm-cli/Cargo.toml` still carries the `[package.metadata.npm]` block (name, prefix, bins, publishConfig access public).
  - Decision `wiki:decisions:remove-self-install-npm-distribution` names npm as the distribution channel.

  ## Change

  Add a `publish-npm` job to `.github/workflows/ci.yml`:

  - runs only on `refs/tags/v*`, needs `publish`, uses the `npm-publish` environment
  - installs cargo-npm
  - downloads the `binary-*` artifacts and places each `wm-cli` binary under `target/<triple>/release`
  - runs `cargo npm generate -p wm-cli --infer-targets`, then `cargo npm publish -p wm-cli` with `NPM_TOKEN`
  - carries nothing from the removed job that belonged to wm-server or the Angular frontend

  ## Out of scope

  Publishing, retagging, or pushing. Those are release decisions for the owner.

  ## Verification

  - ci.yml parses as valid YAML and the job graph is publish -> publish-npm.
  - After the owner pushes a tag, `npm view @something-cabinet/wm-cli version` returns the tagged version and `npm i -g` yields `wm-cli --version` equal to it.

  ## References

  - @wiki/decisions/remove-self-install-npm-distribution
  - @wiki/howto/check-ci-and-npm-status
  - @wiki/specs/windows-release-import-fix
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
  scope: component
  needs_tasks: true