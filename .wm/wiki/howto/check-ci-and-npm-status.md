---
title: 'Howto: Check CI and npm publish status'
id: wiki:howto:check-ci-and-npm-status
type: howto
relates_to:
  - {type: references, target: wiki:patterns:cargo-npm-github-actions}
---

---
title: Howto: Check CI and npm publish status
type: howto
id: wiki:howto:check-ci-and-npm-status
tags: [howto, ci, npm, github-actions, debugging]
---

## Checking CI and npm publish status

### GitHub Actions

After pushing a tag (e.g., `v0.3.0`), the CI workflow runs automatically. To check status:

```bash
# List recent runs (shows last 3)
gh run list --repo something-cabinet/wiki-mem -L 3

# View a specific run
gh run view <run-id> --repo something-cabinet/wiki-mem

# View logs for a failed job
gh run view --job=<job-id> --repo something-cabinet/wiki-mem --log-failed

# View full job logs
gh run view --job=<job-id> --repo something-cabinet/wiki-mem --log
```

The workflow has three job stages:
1. **check** — tests
2. **publish** — matrix build (4 platforms: linux-x64, linux-arm64, darwin-arm64, win32-x64)
3. **publish-npm** — publishes to npm

### npm registry

After publish-npm completes, verify the package is live:

```bash
# Check version on npm
npm view @something-cabinet/wm-cli version

# Check all published versions
npm view @something-cabinet/wm-cli versions

# Check platform-specific package
npm view @something-cabinet/wm-cli-darwin-arm64 version
```

### Common CI failures

| Error | Likely cause | Fix |
|-------|-------------|-----|
| `SIGILL` (signal 4) | CPU lacks AVX2 — prebuilt `libonnxruntime.a` | Run tests without `onnx` feature |
| `aarch64-linux-gnu-gcc: not found` | Missing cross-compiler | Install `gcc-aarch64-linux-gnu` |
| `incompatible with elf64-x86-64` | Wrong linker for ARM64 cross-compile | Set `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER` |
| `npm ERR! 403 Forbidden` | Token lacks publish permission | Use Automation token (with 2FA) or check token expiry |
| `npm ERR! 404 Not Found - PUT` | Token can't create new packages | Create a new Automation token on npm |
| `Package version must be a valid semantic version` | Wrong package.json in publish dir | Ensure `cargo npm generate` was run first |
| `runs-on: Unexpected value ''` | Matrix target vs include mismatch | Ensure `target` and `include` entries match exactly |

### Links

- GitHub Actions: https://github.com/something-cabinet/wiki-mem/actions
- npm package: https://www.npmjs.com/package/@something-cabinet/wm-cli