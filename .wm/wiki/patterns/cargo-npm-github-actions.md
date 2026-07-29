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