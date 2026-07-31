---
title: wm-server npm publish: bundle web UI via direct npm publish
type: spec
id: wiki:specs:wm-server-npm-bundle-publish-fix
status: approved
tags: [spec, ci, npm, release]
---

# wm-server npm publish: bundle web UI via direct npm publish

## Overview

`@something-cabinet/wm-server` platform packages published at v0.3.5 and v0.3.6 contain **no web UI**. `wm-cli web` reports "Web UI not built" because the Angular frontend never reaches the published tarball.

## Root cause

cargo-npm 0.1.2 (`cargo npm publish`) builds each platform tarball from an **explicit entry list** in `publish.rs::pack_platform_package`:

- `package.json`
- each configured `bins` entry (e.g. `wm-server`)
- extra files from `npm::list_extra_files` — auto-discovery of LICENSE/README only

Any file/dir copied into the generated package dir (`npm/@something-cabinet/wm-server-*/wm-web/dist/browser/`) is **silently omitted** from the tarball. The CI "Bundle Angular frontend" step therefore appeared green while shipping nothing.

Verified evidence:
- Published v0.3.6 `wm-server-darwin-arm64` tarball: exactly 2 files (`package.json`, `wm-server`)
- `npm pack` on the same bundled dir: 10 files including `wm-web/dist/browser/index.html`

## Locked Decisions

- D1: Replace `cargo npm publish -p wm-server` with direct `npm publish` per platform package dir (npm packs the whole dir, including `wm-web/`).
- D2: Keep `cargo npm publish -p wm-cli` unchanged (wm-cli needs no extra files).
- D3: Guard the bundle step — fail the build if the glob matches 0 dirs or `index.html` is absent after copy.
- D4: Do not touch unrelated in-progress work (wm-cli-web-review-fixes).

## Requirements

### FR-1: Direct publish for wm-server
- Replace `cargo npm publish -p wm-server` with a loop over `npm/@something-cabinet/wm-server-*/` dirs running `npm publish` in each.
- Publish the main `@something-cabinet/wm-server` package after platform packages (optionalDependencies must resolve).

### FR-2: Bundle step guard
- `[ -d "$dir" ] || { echo "no platform packages found"; exit 1; }` before the copy loop.
- After copy: assert `[ -f "$dir/wm-web/dist/browser/index.html" ]`, else exit 1.

### FR-3: wm-cli unchanged
- `cargo npm publish -p wm-cli` stays as-is.

## Acceptance Criteria

- [ ] AC-1: `npm pack --dry-run` on a bundled platform dir includes `wm-web/dist/browser/index.html` (verified locally — 10 files vs 2 before)
- [ ] AC-2: ci.yml publishes wm-server via direct `npm publish`, not `cargo npm publish -p wm-server`
- [ ] AC-3: Bundle step exits 1 when glob matches 0 dirs or index.html missing
- [ ] AC-4: `cargo npm publish -p wm-cli` still present
- [ ] AC-5: wm-cli-web-review-fixes working-tree changes untouched

## References

- @wiki/tasks/fix-wm-server-npm-packages-ship-without-bundled-web-ui-cargo-npm-drops-wm-web
- @wiki/concepts/cargo-npm-scoped-output-silent-noop-glob (documented only the glob issue; the deeper cargo-npm entry-list behavior was the real blocker)
- @wiki/howto/check-ci-and-npm-status
