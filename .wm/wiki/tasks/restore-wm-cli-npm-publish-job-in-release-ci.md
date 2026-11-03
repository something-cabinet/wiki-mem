---
title: Restore wm-cli npm publish job in release CI
type: task
id: "wiki:tasks:restore-wm-cli-npm-publish-job-in-release-ci"
status: in-progress
priority: high
tags: [ci, release, npm]
spec: wiki:specs:restore-wm-cli-npm-publish
acceptance_criteria:
  - text: "ci.yml has a publish-npm job gated on v* tags that needs publish"
    checked: true
  - text: "The job publishes only wm-cli and its platform packages, with no wm-server or Angular steps"
    checked: true
  - text: "ci.yml parses as valid YAML"
    checked: true
  - text: "A pushed tag results in npm view @something-cabinet/wm-cli version returning the tagged version"
implementation_notes: '2026-11-03: v0.6.1 retagged at 425cd71. check and all four publish jobs passed; publish-npm failed on the first PUT (E404, @something-cabinet/wm-cli-darwin-arm64@0.6.1). Packages exist at 0.5.2 under maintainer vppp. NPM_TOKEN in environment npm-publish was last set 2026-07-29 and the last npm publish was 0.5.2 on 2026-08-17. Likely an expired or under-scoped token; needs a new token from the npm account. Nothing was published at 0.6.1, so a job rerun is clean.'
---

Commit aa49715 deleted the publish-npm job from .github/workflows/ci.yml while retiring wm-server. Tags v0.6.0 and v0.6.1 built binaries that were discarded, so npm latest is still 0.5.2 and the connected MCP server runs 0.5.2. Add a wm-cli-only publish-npm job per the spec.