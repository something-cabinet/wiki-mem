---
implementation_notes: '**Update (v0.3.6 evidence):** The scoped-glob fix alone was NOT sufficient. v0.3.6 CI ran with `npm/@something-cabinet/wm-server-*/` and the copy succeeded, but the published tarball still contained only `package.json` + `wm-server` (verified via `npm pack`/tarball inspection: 2 files, no `wm-web/`). Deeper root cause: cargo-npm 0.1.2 `pack_platform_package` (src/publish.rs) builds platform tarballs from an EXPLICIT entry list — `package.json` + configured `bins` + auto-discovered LICENSE/README (`npm::list_extra_files`) — and never packs arbitrary files copied into the generated package dir. The bundle step can pass green while shipping nothing. Real fix (task: fix-wm-server-npm-packages-ship-without-bundled-web-ui): publish wm-server platform packages via direct `npm publish` (packs the whole dir, incl. `wm-web/`) instead of `cargo npm publish -p wm-server`; guard the bundle loop with `[ -d "$dir" ]` and an `index.html` presence assert.'
---

---
{}
relates_to:
  - {type: references, target: wiki:tasks:bundle-angular-frontend-with-wm-server-for-npm-distribution}
---

---
title: Failure: cargo-npm scoped output dir makes non-matching glob silently skip
type: concept
id: wiki:concepts:cargo-npm-scoped-output-silent-noop-glob
tags: [failure, ci, cargo-npm, glob, npm]
---

# Failure: cargo-npm scoped output dir makes non-matching glob silently skip

## What went wrong
v0.3.5 published `@something-cabinet/wm-server` platform packages WITHOUT the Angular frontend. `wm-cli web` still showed "Web UI not built". CI reported every step green, including "Bundle Angular frontend into wm-server platform packages".

## Root cause
`cargo-npm generate` writes platform packages to a **scoped subdirectory**: `npm/@something-cabinet/wm-server-darwin-arm64/`, not `npm/wm-server-darwin-arm64/`. The bundle step globbed `npm/wm-server-*/`, which matched **zero directories**. In bash, a for-loop over a non-matching glob skips the body silently — no error, no output. The step "succeeded" while copying nothing.

Two compounding surprises:
1. cargo-npm's out-dir layout nests under `@scope/` even when `prefix` is set (docs show flat `npm/my-tool-*/` — misleading).
2. A glob that matches nothing is not a failure — the loop just doesn't iterate. Nothing in CI signals "this glob matched 0 dirs".

## Prevention
- Use the scoped glob: `npm/@something-cabinet/wm-server-*/`
- Guard the loop: `for dir in npm/@something-cabinet/wm-server-*/; do [ -d "$dir" ] || { echo "no platform packages found"; exit 1; }; ...`
- After the first release with a new bundle step, download the published tarball and verify the bundled assets are present — the symptom (missing UI) only shows at runtime for users
- Alternatively assert on the copy target: `[ -f "$dir/wm-web/dist/browser/index.html" ]` at the end of the loop

## Time lost
~1 release cycle (v0.3.6) + user-side confusion debugging "Web UI not built".

## Related
- @wiki/tasks/bundle-angular-frontend-with-wm-server-for-npm-distribution
- @wiki/patterns/cargo-npm-github-actions