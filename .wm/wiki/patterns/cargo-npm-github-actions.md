---
{}
relates_to:
  - {type: references, target: wiki:tasks:bundle-angular-frontend-with-wm-server-for-npm-distribution}
---

---
{}
relates_to:
  - {type: references, target: wiki:core:README}
---

---
title: Pattern: cargo-npm + GitHub Actions for multi-platform Rust CLI distribution
type: pattern
id: wiki:patterns:cargo-npm-github-actions
status: draft
tags: [pattern, ci, rust, npm, github-actions]
---

## Problem
Distributing a Rust CLI binary across platforms (Linux, macOS, Windows) requires building for each target and hosting the binaries somewhere users can easily download and install.

## Solution
Use `cargo-npm` to package compiled Rust binaries as platform-specific npm packages, published via a GitHub Actions matrix build.

### Setup

1. **Add cargo-npm config** to the crate's `Cargo.toml`:

```toml
[package.metadata.npm]
name = "@scope/my-cli"
prefix = "@scope/my-cli-"
bins = ["my-cli"]
custom = { publishConfig = { access = "public" } }
```

2. **GitHub Actions matrix**: Build for each target in parallel:

```yaml
publish:
  strategy:
    matrix:
      target:
        - x86_64-unknown-linux-gnu
        - aarch64-unknown-linux-gnu
        - aarch64-apple-darwin
        - x86_64-pc-windows-msvc
```

3. **Cross-compilation for ARM64 Linux** requires `gcc-aarch64-linux-gnu` and linker config:

```yaml
- name: Install cross-compiler for aarch64-unknown-linux-gnu
  if: matrix.target == 'aarch64-unknown-linux-gnu'
  run: sudo apt-get install -y gcc-aarch64-linux-gnu g++-aarch64-linux-gnu

- name: Build release
  run: cargo build --release -p wm-cli --target ${{ matrix.target }}
  env:
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER: aarch64-linux-gnu-gcc
    CC_aarch64_unknown_linux_gnu: aarch64-linux-gnu-gcc
    PKG_CONFIG_ALLOW_CROSS: 1
```

4. **Each matrix job uploads only the binary** as an artifact, then a single publish job collects all binaries and runs `cargo npm generate --infer-targets` to create all platform packages with correct dependency references.

5. **Publish**: Use `cargo npm publish` with an npm automation token (requires 2FA enabled on npm account) stored as a GitHub secret.

### Multi-binary distribution: one npm package per crate

`cargo-npm` `bins` ONLY accepts binaries from the SAME crate. `bins = ["my-cli", "my-server"]` fails with `error: unknown bin(s) ["my-server"]; available: ["my-cli"]`.

To ship multiple binaries (e.g. a CLI + a server daemon):

1. Add `[package.metadata.npm]` to EACH crate (both get their own platform packages)
2. Reference the secondary package as an `optionalDependencies` of the main one:

```toml
# main crate Cargo.toml
[package.metadata.npm]
name = "@scope/my-cli"
prefix = "@scope/my-cli-"
bins = ["my-cli"]
custom = {
  publishConfig = { access = "public" },
  optionalDependencies = { "@scope/my-server" = "^0.3" }
}
```

3. At runtime, resolve the sibling binary by walking up from `current_exe()` scanning `node_modules/@scope/my-server-*/` directories — handles both hoisted and nested npm layouts.
4. CI: `cargo npm generate -p my-cli` AND `-p my-server`, then publish both.

### Bundling a web frontend into the binary package

When the CLI serves a web UI (e.g. `wm-cli web` → Axum server + Angular SPA):

1. Build the frontend in the `publish-npm` job (needs `actions/setup-node@v4`, check the Angular CLI's Node minimum — Angular 17+ needs Node 22+):

```yaml
- name: Set up Node.js for frontend build
  uses: actions/setup-node@v4
  with:
    node-version: 22
    cache: npm
    cache-dependency-path: apps/wm-web/package-lock.json

- name: Build Angular frontend
  run: |
    cd apps/wm-web
    npm ci
    npm run build
```

2. After `cargo npm generate`, copy the built assets into each server platform package next to the binary:

```yaml
- name: Bundle frontend into server platform packages
  run: |
    for dir in npm/wm-server-*/; do
      mkdir -p "$dir/wm-web/dist/browser"
      cp -r apps/wm-web/dist/browser/* "$dir/wm-web/dist/browser/"
    done
```

3. Serve via `tower-http::services::ServeDir` with an `index.html` fallback for client-side routing.
4. Runtime lookup: `exe.parent()/wm-web/dist/browser` relative to the server binary — no additional install steps for users.

⚠️ Angular 17+ application builder outputs to `dist/browser/`, NOT `dist/` directly. Check both paths when locating `index.html`.

### Auth

- Create an **Automation token** on npm (requires 2FA) — no expiry, bypasses 2FA for CI
- Store as `NPM_TOKEN` GitHub secret
- Configure `.npmrc` in the publish step before running cargo-npm

### Users install with

```bash
npm install -g @scope/my-cli
```

## When to Use
- Distributing Rust CLI binaries to users who have Node.js/npm installed
- Projects needing multi-platform distribution without managing binary hosting
- When `cargo install` is undesirable (slow compilation, Rust toolchain requirement)

## When Not to Use
- Users are exclusively Rust developers (use `cargo install` via crates.io)
- The binary has system-specific native dependencies beyond what cross-compilation handles
- npm is not available on target systems

## Related
- GitHub: https://github.com/abemedia/cargo-npm
- @wiki/tasks/bundle-angular-frontend-with-wm-server-for-npm-distribution
- @wiki/memory/wm-cli-web-must-bundle-wm-server-in-npm-package