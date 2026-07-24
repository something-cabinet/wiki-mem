---
id: wiki:specs:code-intel-search-ui
title: Code Intel Search Page — Web UI
type: spec
tags: [spec, code-intel, web-ui, angular, search]
status: draft
---
id: wiki:specs:code-intel-search-ui

# Code Intel Search Page — Web UI

## Overview

Add a **Code Intelligence** view to wm-web that exposes tree-sitter-based code search, symbol lookup, and dependency analysis through a dedicated UI — complementing the existing wiki search with code-aware queries. The existing `wm-code-intel` crate already extracts symbols and dependencies for Rust, TypeScript (including TSX), Python, Go, HTML, and Svelte files. This spec adds the frontend to make it discoverable and usable from the browser.

## Prerequisites (Backend) — COMPLETED

1. **Enable `code-intel` feature in `wm-server`**: ✅ Done. Added to `apps/wm-server/Cargo.toml`.
2. **Add `wm_code.file` tool**: ✅ Done. New MCP tool reads file content confined to project root, with directory traversal protection and dotfile rejection.
3. **Add `file` param to `wm_code.symbols`**: ✅ Done. Added `WmCodeSymbolsInput.file: Option<String>` for per-file symbol listing.
4. **Add reverse dep inversion to `wm_code.deps`**: ✅ Done. Added `WmCodeDepsInput.reverse: Option<bool>`.
5. **Add `max_results` param to `wm_code.symbols`**: ✅ Done. Applied via truncation after collection. Early-exit not yet implemented (future optimization).
6. **Dep target → file resolution**: ⏳ Best-effort heuristic REMOVED from initial scope. Reverse deps are computed via string matching (module specifier vs file path). Known limitation: module strings like `crate::foo` may not resolve to actual file paths.

## API Strategy

Use the existing `/api/tools` generic dispatch (calls `POST /api/tools/{name}` which matches any registered MCP tool).

### Tool Reference

| Tool | Params | Response | Notes |
|------|--------|----------|-------|
| `wm_code.symbols` | `{name?, kind?, path?, language?, file?, max_results?}` | `{symbols: CodeIntelSymbol[]}` | Symbol search by name (case-sensitive substring). `path` is a subdirectory filter; `file` is per-file filter. |
| `wm_code.deps` | `{file?, reverse?}` | `{dependencies: {file, deps: CodeIntelDep[]}}` | Forward deps per file. `reverse: true` inverts the scan. |
| `wm_code.file` | `{path}` | `{content: string, language: string}` | Read file content, confined to project root. Rejects dotfiles and hidden directories. |

### Data Types

```typescript
interface CodeIntelSymbol {
  name: string;
  kind: 'function' | 'struct' | 'enum' | 'trait' | 'class' | 'interface' | 'type' | 'const' | 'module' | 'method' | 'macro' | 'impl';
  file: string;
  line: number;
  column: number;
  snippet: string;
  parent_name: string;
  language: string;
}

interface CodeIntelDep {
  target: string;
  kind: 'use' | 'import';
  line: number;
}

interface CodeIntelDepSet {
  file: string;
  deps: CodeIntelDep[];
}
```

## Requirements

### FR-1: Navigation
- New sidebar nav item "Code" with `lucideCode` icon between "Graph" and "Tasks"
- Route: `/code`
- Active route highlighting consistent with existing nav (prefix matching)

### FR-2: Symbol Search
- Search input with 300ms debounced auto-search
- Search across code symbols by name filter via `wm_code.symbols`
- Minimum query length: 2 characters (avoids triggering full-project scans)
- Results grouped by language
- Each result shows: symbol name, kind (badge), file path, line number
- Click on result → show symbol detail panel
- Results capped client-side at 200; `max_results` param passed to backend

### FR-3: Symbol Detail Panel
- Symbol name and kind badge (color-coded by kind: blue for functions, purple for classes, etc.)
- File path with line number
- Parent name (if applicable)
- Source code snippet from the symbol definition
- Back button returns to search results
- Open file button → load file in file browser

### FR-4: File Code Browser
- Shows file content in a monospace pre block
- Lists all symbols in the file with line numbers
- File path as header, relative to project root
- Click a symbol in the file → navigate to symbol detail
- Paths confined to the project root; dotfiles rejected (security)

### FR-5: Dependencies
- For any file, show its direct imports/uses
- Collapsible sections
- Dep targets shown as plain text

### FR-6: Language Filter
- Pills: All, Rust, TypeScript/TSX, Python, Go, HTML, Svelte
- TypeScript and TSX grouped under same pill
- Persisted in component state during session

### FR-7: Empty/Loading/Error States
- Initial: Search icon + prompt
- No results: "No symbols found" with suggestion
- Error: alert with retry button
- Loading: spinner

## Acceptance Criteria

- [ ] AC-1: `/code` route exists and nav item "Code" in sidebar
- [ ] AC-2: Symbol search returns results grouped by language, debounced at 300ms
- [ ] AC-3: Minimum 2-character query before search triggers
- [ ] AC-4: Clicking a symbol shows detail with kind badge, file path, snippet, deps
- [ ] AC-5: File browser loads file content and lists all file symbols
- [ ] AC-6: Deps tab shows imports from the file
- [ ] AC-7: Language filter pills work and persist during session
- [ ] AC-8: Empty/loading/error states for all states
- [ ] AC-9: Backend tools reachable via `/api/tools` dispatch
- [ ] AC-10: `code-intel` feature enabled in wm-server
- [ ] AC-11: Mobile responsive layout
- [ ] AC-12: CodeIntelPort + MockCodeIntelService compile without errors

## References

- `packages/wm-code-intel/` — symbol extraction engine
- `apps/wm-core/src/mcp/tools/code.rs` — MCP code tool implementations
- `apps/wm-web/src/app/views/code/` — CodeView component
- `apps/wm-web/src/app/services/code-intel-port.ts` — CodeIntelPort interface
