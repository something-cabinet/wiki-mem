---
title: 'Howto: Check CI and npm publish status'
id: wiki:howto:check-ci-and-npm-status
type: howto
tags: [howto, ci, npm, github-actions, debugging]
relates_to:
  - {type: references, target: wiki:patterns:cargo-npm-github-actions}
  - {type: references, target: wiki:tasks:restore-wm-cli-npm-publish-job-in-release-ci}
---

schema_version: 1
state: |-
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
  1. **check** - tests
  2. **publish** - matrix build (4 platforms: linux-x64, linux-arm64, darwin-arm64, win32-x64)
  3. **publish-npm** - publishes to npm

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

  Right after a release, npm may serve cached metadata. Add `--prefer-online` to `npm view` and `npm install`. An install from stale metadata skips the platform package and `wm-cli` then crashes with `Cannot find module '@something-cabinet/wm-cli-darwin-arm64/wm-cli'`.

  ### Trusted publishing (current auth for publish-npm)

  publish-npm authenticates with GitHub OIDC, not NPM_TOKEN. The job needs `permissions: id-token: write` and Node 24 (npm 11.5.1 or newer). v0.6.2 is the first release published this way.

  Each of the five packages needs its own trusted publisher with an allowed action: `@something-cabinet/wm-cli`, `-darwin-arm64`, `-linux-arm64`, `-linux-x64`, `-win32-x64`. Since 2026-05-20 npm requires every trusted publisher to name at least one allowed action. Create them with npm 11.15.0 or newer:

  ```bash
  npx -y npm@11.19.0 trust github @something-cabinet/<package> --repository something-cabinet/wiki-mem --file ci.yml --environment npm-publish --allow-publish --yes
  ```

  - A config with no allowed action matches the repo, workflow and environment but permits nothing. The publish then fails with `403 OIDC permission denied for this action`.
  - npm 11.14.1 and older send no permissions field, so `npm trust github` ends in a bare `400 Bad Request`. This machine had 11.12.1.
  - Each `trust` call needs a security key check in the browser, so run it in your own terminal.
  - Replace an existing config by running `npm trust revoke <package> --id=<id>` first. `npm trust list <package>` shows the ids.
  - Verify all five with `npm trust list <package>` before pushing a release tag.
  - `cargo npm publish` uploads the platform packages one at a time and npm never allows a version to be published twice. A tag run that reaches the registry with some packages untrusted can leave a half-published version, which forces a version bump.
  - A run that fails on the first upload leaves nothing published, so `gh run rerun <run-id> --failed` is safe, provided the tagged commit already holds the correct workflow file.
  - The `wm-server*` packages are retired. CI never publishes them, so they need no trusted publisher. They are deprecated with `npm deprecate "@something-cabinet/wm-server*@*" "wm-server was retired in 0.6. Use @something-cabinet/wm-cli."`, which needs a security key check.
  - A repo with `use_immutable_subject` true in its OIDC customization sends an id-based `sub` claim. That did not block the publish.

  ### Common CI failures

  | Error | Likely cause | Fix |
  |-------|-------------|-----|
  | `SIGILL` (signal 4) | CPU lacks AVX2 - prebuilt `libonnxruntime.a` | Run tests without `onnx` feature |
  | `aarch64-linux-gnu-gcc: not found` | Missing cross-compiler | Install `gcc-aarch64-linux-gnu` |
  | `incompatible with elf64-x86-64` | Wrong linker for ARM64 cross-compile | Set `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER` |
  | `npm ERR! 403 Forbidden` | Token lacks publish permission | Check token permissions and expiry |
  | `404 Not Found - PUT` on a package that already exists | Token expired, revoked or lacking write. npm answers 404 instead of 401 | Run `npm token list --json` and read `expiry` and `revoked`. Granular write tokens last at most 90 days. Prefer trusted publishing |
  | `403 ... OIDC permission denied for this action` | The package's trusted publisher has no allowed action, or its repository, workflow file or environment differs from the run | Recreate it with `--allow-publish` (see above). Workflow is the file name `ci.yml`, not a path |
  | `400 Bad Request` from `npm trust github` with no body | npm older than 11.15.0 | Use `npx -y npm@11.19.0 trust github ...` |
  | `Cannot find module '@something-cabinet/wm-cli-<platform>/wm-cli'` after a fresh install | Install used stale registry metadata and skipped the platform package | Reinstall with `--prefer-online` |
  | `Package version must be a valid semantic version` | Wrong package.json in publish dir | Ensure `cargo npm generate` was run first |
  | `runs-on: Unexpected value ''` | Matrix target vs include mismatch | Ensure `target` and `include` entries match exactly |

  ### Links

  - GitHub Actions: https://github.com/something-cabinet/wiki-mem/actions
  - npm package: https://www.npmjs.com/package/@something-cabinet/wm-cli
questions:
  - id: task_kind
    type: choice
    instructions: What kind of task does this howto cover?
    options:
    - setup
    - development
    - testing
    - release
    - debugging
    - operations
    - integration
  - id: prerequisites_complete
    type: noul
    instructions: All prerequisites for this howto are fully documented.
  - id: difficulty
    type: score
    instructions: What skill level does this howto require?
    levels:
    - beginner
    - intermediate
    - advanced
    - expert
  - id: has_verification
    type: noul
    instructions: This howto includes a verification step.
answers: {}