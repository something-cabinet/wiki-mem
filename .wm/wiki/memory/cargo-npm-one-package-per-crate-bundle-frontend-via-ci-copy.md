---
title: cargo npm one package per crate bundle frontend via ci copy
id: wiki:memory:cargo-npm-one-package-per-crate-bundle-frontend-via-ci-copy
type: memory
tags: [npm, ci, cargo-npm, frontend, bundling]
---

To ship multiple binaries via cargo-npm, each crate needs its own [package.metadata.npm] — `bins` only accepts same-crate binaries. Reference secondary packages as optionalDependencies. To serve a web UI from a Rust binary npm package, build the frontend in the publish-npm job (Node 22 for Angular 17+), copy dist/browser into each server platform package, and serve with ServeDir + index.html fallback.

WARNING: cargo-npm writes packages to SCOPED dirs (npm/@scope/my-server-*/) — globbing npm/my-server-*/ matches nothing and silently skips (v0.3.5 shipped without frontend). Full reference: @wiki/patterns/cargo-npm-github-actions