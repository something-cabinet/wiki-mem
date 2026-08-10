---
title: 'Fix: wm-server npm packages ship without bundled web UI (cargo-npm drops wm-web)'
id: wiki:tasks:fix-wm-server-npm-packages-ship-without-bundled-web-ui-cargo-npm-drops-wm-web
type: task
status: done
priority: high
tags:
- from-spec
- ci
- npm
- release
- finding
implementation_notes: 'Implemented in .github/workflows/ci.yml: (1) bundle step guards `[ -d "$dir" ]` and asserts `[ -f "$dir/wm-web/dist/browser/index.html" ]` after copy, exits 1 otherwise; (2) publish step replaces `cargo npm publish -p wm-server` with direct `npm publish` loop over `npm/@something-cabinet/wm-server-*/` dirs, then publishes main `@something-cabinet/wm-server` package. Verified locally: regenerate + guarded bundle + `npm pack --dry-run` → 10 files incl. wm-web/dist/browser/index.html (published 0.3.6 tarball was 2 files, no wm-web). Negative test: glob matching 0 dirs exits 1 under bash. wm-cli publish unchanged.'
acceptance_criteria:
- text: npm pack --dry-run on a bundled platform dir includes wm-web/dist/browser/index.html (verified locally)
---

Published @something-cabinet/wm-server platform packages (v0.3.5, v0.3.6) contain only package.json + wm-server binary. The Angular frontend copied into the generated package dir by CI is silently dropped because cargo-npm 0.1.2 builds platform tarballs from an explicit entry list (package.json + bins + license/readme), never packing arbitrary copied files. Verified: published 0.3.6 tarball = 2 files; npm pack on the same dir = 10 files incl. wm-web/. Fix: publish wm-server packages via direct npm publish (packs whole dir) instead of cargo npm publish, plus guard the bundle step to fail when index.html is missing.