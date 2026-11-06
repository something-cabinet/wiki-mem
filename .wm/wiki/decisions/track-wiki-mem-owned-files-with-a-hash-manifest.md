---
title: 'Decision: Track wiki-mem-owned project files with a hash manifest'
type: decision
id: "wiki:decisions:track-wiki-mem-owned-files-with-a-hash-manifest"
status: draft
relates_to:
  - {type: implements, target: wiki:specs:wm-cli-migrate-upgrades-the-whole-wiki-with-versioned-records}
  - {type: references, target: wiki:tasks:add-wm-cli-migrate-migrate-the-whole-wiki-with-versioned-records}
---

schema_version: 1
state: |-
  ## Context

  `wm-cli setup`, `init` and `agents --sync` write skills, the Kiro steering file, `WIKI-MEM.md` and the agent shims into a project. They overwrote those files on every run and recorded nothing about where a file came from. `wm-cli migrate` only upgraded wiki pages and legacy memory, so an old skill or steering file could not be detected, and an edited one could be lost.

  ## Decision

  Write `.wm/managed-files.json`, mapping each managed file to `{version, sha256}` of what wiki-mem wrote. `wm-cli migrate` classifies every existing managed file:

  - Matches the template and the manifest: up to date.
  - Matches the template, no matching manifest entry: recorded.
  - Matches the manifest but not the template: replaced with the current template.
  - Differs from the manifest: edited, kept and reported.
  - No manifest entry and different from the template: unmanaged, kept and reported.

  Skills, the steering file and `WIKI-MEM.md` are hashed as whole files. The shims are hashed on the `WIKI-MEM GUIDELINES START/END` block only, and updates splice that block, because `setup` appends an import line after the block. `setup` and `agents --sync` record files after writing. Edited files are skipped, with no `--force` option (decided by the maintainer).

  ## Rationale

  A manifest works for every file type and detects edits. An in-file version key was rejected: JSON configs have no safe place for one, and skills and steering are read by other tools. Hashing whole shims reported `AGENTS.md` as edited right after `setup`, which is why shims use the block.

  ## Consequences

  - MCP JSON configs are not covered, because they are merged into files the user also edits.
  - Installs from before this change have no manifest. Files equal to the template get recorded, and different ones are reported as unmanaged and left alone.
  - Edited and unmanaged files do not fail the `migrate` exit code.
  - Commit `c3d16f8`.

  ## Related

  - `wiki:specs:wm-cli-migrate-upgrades-the-whole-wiki-with-versioned-records`

questions:
  - id: outcome
    type: choice
    instructions: What is the recorded outcome of this decision?
    options:
    - adopted
    - rejected
    - deferred
    - superseded
    - abandoned
  - id: reversibility
    type: noul
    instructions: The decision can be reversed cheaply without data migration or cross-module breakage.
  - id: confidence
    type: score
    instructions: How strong is the recorded justification for the selected outcome?
    levels:
    - low
    - medium
    - high
  - id: impact
    type: choice
    instructions: How wide is the blast radius of this decision?
    options:
    - local
    - component
    - system
    - project-wide
answers: {}