---
implementation_plan: '## Implementation Plan ### Approach: Ship dist/ alongside binary in npm platform package (Option 2) Rationale: rust-embed bloats every binary per-platform (~+2-5MB each, 4 targets) and lengthens compile; a separate wm-web npm package needs runtime discovery logic and version coupling. Shipping `dist/browser/` next to `wm-server` reuses the existing `spa::find_dir()` priority-2 lookup (`exe.parent()/wm-web/dist/browser`) with zero Rust changes. ### Steps 1. **CI: Add frontend build to `publish-npm` job** (`.github/workflows/ci.yml`) - Add `actions/setup-node@v4` with `node-version: 22` (Angular CLI requires v22+) - `npm ci` in `apps/wm-web/`, then `npm run build` (output: `apps/wm-web/dist/browser/`) 2. **CI: Copy dist into each generated platform package before publish** - After `cargo npm generate -p wm-cli -p wm-server --infer-targets`: - Loop `npm/wm-server-*/`, copy `apps/wm-web/dist/browser/` → `npm/wm-server-{platform}/wm-web/dist/browser/` - Confirms `spa::find_dir()` priority-2 path (`current_exe().parent()/wm-web/dist/browser`) resolves 3. **Verify locally (before tagging)** - Run the generate + copy flow manually; inspect `npm/wm-server-darwin-arm64/` contents include `wm-web/dist/browser/index.html` - `cargo build -p wm-server`, start `./target/debug/wm-server`, `curl localhost:4090/` returns Angular HTML (dev path already works — confirm bundled path) 4. **Docs** - README CLI table: `wm web` note — serves full web UI when bundled frontend present - Document binary-package layout and that no extra npm install is needed ### Validation - `wm-cli web` serves `http://localhost:4090/` Angular UI from npm install (AC-1) - Monorepo dev flow (`cargo build` + `ng build`) unchanged (AC-2) - Angular build in CI adds ~1-3 min on one runner (AC-3) - Record bundle size: `wm-server` binary unchanged; `wm-web/dist` (~1-3MB) added to each platform package (AC-4) ### Out of scope - rust-embed embedding (Option 1) and separate wm-web package (Option 3) — documented in task, not implemented'
status: in-progress
---

---
title: Bundle Angular frontend with wm-server for npm distribution
type: task
tags: [frontend, npm, ci, deployment]
status: in-progress
priority: medium
assignee: @me
acceptance_criteria:
  - {text: "npm install -g @something-cabinet/wm-cli && wm-cli web serves the Angular UI at http://localhost:4090 without any additional build steps", checked: false}
  - {text: "Development workflow (cargo build + ng build from monorepo) still works", checked: false}
  - {text: "CI build time is not excessive (< 3 minutes added)", checked: false}
  - {text: "Binary size increase is documented and reasonable", checked: false}
---

The wm-server now serves the Angular SPA when built at apps/wm-web/dist/browser/. For npm-distributed versions, the frontend needs to be built and bundled with the binary so wm-cli web serves the full UI out of the box.

Options to investigate:
1. Build Angular app in CI and embed static files into wm-server binary using rust-embed (self-contained, no extra files to ship)
2. Build Angular in CI and ship the dist/ alongside the binary in the npm platform package (standard file serving via ServeDir)
3. Build Angular and publish as a separate @something-cabinet/wm-web npm package, with wm-server discovering it at runtime