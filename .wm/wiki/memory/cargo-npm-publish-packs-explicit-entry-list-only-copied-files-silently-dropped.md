---
title: cargo-npm publish packs explicit entry list only — copied files silently dropped
type: memory
tags: [cargo-npm, npm, ci, release, failure]
status: active
---

When publishing Rust CLIs as npm platform packages with cargo-npm 0.1.2, `cargo npm publish` builds each platform tarball from an explicit entry list: package.json + configured bins + auto-discovered LICENSE/README files (src/publish.rs::pack_platform_package, src/npm.rs::list_extra_files). ANY other file or dir copied into the generated package dir (e.g. bundling an Angular dist into npm/@scope/pkg-*/wm-web/) is SILENTLY omitted from the tarball — CI bundle steps pass green while shipping nothing. Fix pattern: publish the platform dirs with direct `npm publish` (packs the whole dir) instead of `cargo npm publish -p <crate>`; guard bundle loops with a dir-exists check and an index.html presence assert. Repro check: `npm pack --dry-run` on the generated dir shows 10 files vs the 2 in the published tarball.