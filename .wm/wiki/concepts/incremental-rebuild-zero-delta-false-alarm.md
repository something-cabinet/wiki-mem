---
title: 'Failure: Incremental rebuild zero-delta false alarm'
id: wiki:concepts:incremental-rebuild-zero-delta-false-alarm
type: concept
relates_to:
  - {type: relates_to, target: wiki:patterns:hash-skip-rebuild}
---

---
{}
relates_to:
  - {type: references, target: wiki:tasks:wm-index-code-output-misleading--report-totals-make---skip-hash-check-force-re-parse}
---

---
title: Failure: Incremental rebuild zero-delta false alarm
type: concept
id: wiki:concepts:incremental-rebuild-zero-delta-false-alarm
tags: [failure, debugging, incremental, code-intel]
---

# Failure: Incremental rebuild zero-delta false alarm

## What went wrong

User reported "wm index code returns 0 symbol index" on eightcap-new-portal (published wm-cli 0.3.6). Output: "7230 files scanned, 0 symbols indexed, 0 dependencies indexed", exit 0. Investigation showed the index was healthy: code.db held 7230 files (6394 typescript, 800 html, 36 python), 37354 symbols, 20370 deps. The run reported "0 symbols indexed" because hash-skip found all files unchanged → 0 NEW symbols this run. A delta was printed where users expect a total.

## Root cause

Incremental hash-skip rebuilds report only the changed set. On a no-change run the delta is 0 — indistinguishable from a broken pipeline. Compounding factors:
- The CLI printed the delta as "symbols indexed" (see @wiki/patterns/cli-delta-vs-total-reporting).
- Some files legitimately extract 0 symbols: `jest.config.ts` uses `module.exports = {...}`, an assignment expression not captured by the TS symbol queries (function/method/class/interface/type/enum/abstract_method_signature/variable_declarator). Testing incrementality with a config file proves nothing.

## Prevention

- Report totals + delta ("N symbols in index (+M new)") so no-change runs are unambiguous
- When investigating "0 indexed": query the persisted store directly first — `sqlite3 .wm/state/code.db "SELECT count(*) FROM code_symbols"` — before suspecting the pipeline
- Check the file's constructs against the language's symbol-query coverage; a 0-symbol file may be correct
- Live-verify incrementality with a symbol-rich file: append a newline to a file with `export class` / `pub fn`, re-run, confirm N > 0, then restore via git checkout

## Time lost

~45 min of agent investigation plus user confusion across sessions, all from an output-format bug, not an indexing bug.

## Related

- @wiki/patterns/hash-skip-rebuild
- @wiki/patterns/cli-delta-vs-total-reporting
- @wiki/specs/code-index-cache