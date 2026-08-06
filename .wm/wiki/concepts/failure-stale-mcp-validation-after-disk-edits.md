---
title: 'Failure: MCP Validation Serves Stale Graph Until Index Rebuild'
type: concept
id: wiki:concepts:failure-stale-mcp-validation-after-disk-edits
status: draft
tags:
- failure
- validation
- mcp
- index-rebuild
- staleness
relates_to:
  - {type: references, target: wiki:tasks:fix-readme}
---

## What went wrong

After bulk-editing 230 wiki files on disk (acceptance_criteria backfills + ref re-pointing), `wm_validate.check` returned output **byte-identical to the pre-fix run** (still 236 errors). The CLI-side `wm-cli validate` passed, but the MCP-side validation saw none of the changes. Time was wasted re-diagnosing already-fixed errors.

## Root cause

The MCP server holds an in-memory graph snapshot. File edits on disk are NOT reflected in validation until the index is rebuilt. The "dirty-bit + directory mtime auto-rebuild" (see @wiki/core:architecture) triggers on search/rebuild paths, but `wm_validate.check` served the stale snapshot without triggering a rebuild. Only an explicit `wm_index_rebuild` (skip_embed=true) refreshed the graph; validation then correctly reported 0 errors.

## Prevention

- After any bulk direct-to-disk wiki edit (sed, parallel fixers, scripts), run `wm_index_rebuild` (skip_embed=true) BEFORE calling `wm_validate.check` via MCP.
- Treat a validate result that is byte-identical to a pre-fix run as a staleness signal, not a "fixes didn't land" signal. Compare against the previous run before re-investigating.
- CLI (`wm-cli validate`) builds a fresh engine in-process and reads disk directly — use it as the ground-truth check when MCP results look stale.

## Time lost

~20 minutes diagnosing a phantom failure that was actually a stale MCP snapshot.

## Related

- @wiki/tasks/fix-readme
- @wiki/core:architecture