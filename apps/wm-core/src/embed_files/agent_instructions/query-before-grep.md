# Query-before-grep

This project is a wiki-mem repository: pages, graph edges, and code symbols are
indexed by the wiki-mem engine and exposed through MCP tools (the `wm_*` prefix).

## Rule

Query the engine BEFORE falling back to raw file reads / greps:

1. `wm_initial` — session bootstrap; returns project state, graph stats, and
   the available search modes.
2. `wm_search.query` / `wm_search.retrieve` — keyword/semantic discovery over
   pages, memory, and code symbols; `retrieve` returns a structured context
   pack with citations.
3. `wm_graph.neighbors` / `wm_graph.full` / `wm_graph.path` /
   `wm_graph.affected` — traverse the typed edge graph (wiki refs, typed code
   edges, provenance) instead of grepping for relationships by hand.
4. `wm_code.search` / `wm_code.deps` — AST-aware symbol lookup and dependency
   analysis before falling back to file greps.

Fall back to raw reads/greps only when the engine has no answer (unindexed
files, missing index). Raw greps bypass provenance, ranking, and cross-file
edges — treat them as a last resort, not a first move.
