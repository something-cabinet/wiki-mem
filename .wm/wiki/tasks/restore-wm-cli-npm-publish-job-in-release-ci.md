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
---

Commit aa49715 deleted the publish-npm job from .github/workflows/ci.yml while retiring wm-server. Tags v0.6.0 and v0.6.1 built binaries that were discarded, so npm latest is still 0.5.2 and the connected MCP server runs 0.5.2. Add a wm-cli-only publish-npm job per the spec.