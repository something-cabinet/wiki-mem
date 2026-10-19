---
title: wm-cli page update hides its stdin payload contract, so agents cannot discover how to edit a body
type: task
id: "wiki:tasks:page-update-does-not-document-its-stdin-payload"
status: done
tags: [bugfix, cli, docs, agent-dx]
priority: high
acceptance_criteria:
  - {text: "page update --help documents the stdin JSON payload: the accepted field names and that input comes from stdin", checked: false}
  - {text: "The help states that record-bearing types require a full record envelope in content, not bare prose", checked: false}
  - {text: "A --content-file option (or equivalent) lets a caller pass the body without shell quoting", checked: false}
  - {text: "An agent given only --help can update a spec body successfully on the first attempt", checked: false}
implementation_notes: |-
  Implemented in 03f4e4f (options 1, 2 and 4).

  1. Bare-prose content on a record-bearing page type now returns invalid_params before writing, with an error
  that names the four envelope keys, points at --content-file, and says omitting content preserves the body.
  2. page update gained --content-file <path>; supplying both sources is rejected; the update subcommand's
  --help now lists the stdin fields.
  4. Contract recorded in .wm/AGENTS.md Tool Usage Rules item 6. No wiki howto page, deliberately: that page
  type does not exist on every system, so the binary and AGENTS.md are the portable carriers. The help_page
  golden was regenerated for the new subcommand description.
---

**Problem.** `wm-cli page update <ID>` accepts a JSON object on **stdin** and writes any of `content`,
`status`, `tags`, `title`, `relates_to`, `type`, `extra_frontmatter`, `notes`, `append_notes`
(`apps/wm-cli/src/page/page_service.rs:120-137`, `attach_update_identity`, and `PageUpdateParams`).
None of that is discoverable: `page update --help` prints only `<ID>` and `--json`, because the clap
variant declares only those two (`apps/wm-cli/src/page/page_action_model.rs:31-38`).

**Observed 2026-10-19.** An agent working from `--help` concluded it could not change a spec body at all and
left two requirements unverified rather than update them. The capability was there the whole time.

**Second trap, worth documenting in the same place.** For record-bearing page types the `content` value must
be the full record envelope (`schema_version`, `state: |-`, `questions`, `answers`). Update writes the body
verbatim and does not re-wrap it (`page_update_builder_service.rs:195-209`), so passing bare prose silently
destroys the record envelope, and passing the entire file duplicates the frontmatter block - the corruption
class already recorded in @wiki/core:critical-patterns.

**Fix direction.** Put the payload contract in `--help` and in the tool description: the field list, that the
input is stdin JSON, and the envelope requirement for record-bearing types. A `--content-file` flag would
remove the shell-quoting hazard that makes agents avoid the command in the first place.