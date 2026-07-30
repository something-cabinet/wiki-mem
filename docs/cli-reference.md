# WM CLI Reference

`wm` is the single entry-point binary for the Wiki Memory Engine. All operations are available through its subcommands or via MCP tools (see `wm mcp`).

## Global Options

| Flag | Description |
|------|-------------|
| `--tui` | Force TUI mode (auto-detected when running interactively) |
| `-h, --help` | Print help |
| `-V, --version` | Show version |

## Commands

### `wm init`
Initialize a new `.wm` project in the current directory.

```bash
wm init                                    # Interactive wizard
wm init --no-wizard                        # Headless, use defaults
wm init my-project                         # Name the project
wm init --git-tracked                      # Track .wm/ in git
wm init --git-ignored                      # Add .wm/ to .gitignore
```

### `wm mcp`
Start the MCP (Model Context Protocol) server over stdio. This is how AI agents connect — OpenCode, Claude Code, Kiro, etc.

```bash
wm mcp                                     # Start MCP server
```

The MCP server starts an embedded HTTP API server internally and proxies all tool calls through it. No separate server process needed.

### `wm setup <platform>`
Generate MCP config + sync skills for a specific platform (re-runs init's final step):

```bash
wm setup opencode       # opencode.json + .opencode/skills/
wm setup claude         # .mcp.json + .claude/skills/
wm setup codex          # .codex/config.toml + .codex/skills/
wm setup kiro           # .kiro/settings/mcp.json + .kiro/steering/ + .kiro/skills/
wm setup cursor         # .cursor/mcp.json + .agent/skills/
wm setup antigravity    # .gemini/antigravity/mcp_config.json + .agents/skills/
```

### `wm agents`
Sync/generate agent instruction files and skills.

```bash
wm agents                # Show status of instruction files
wm agents --sync         # Force re-sync all instruction files
```

Skills are synced from the embedded binary to `.claude/skills/`, `.agents/skills/`, and other platform directories.

### `wm tui`
Launch the interactive terminal UI (Ratatui-based).

```bash
wm tui                   # Interactive TUI
```

### `wm search`
Search wiki pages and memory.

```bash
wm search query "query text"         # Hybrid search (keyword + semantic)
wm search query "query" --mode keyword  # Keyword-only mode
wm search query "query" --json         # JSON output
wm search query "query" --type page    # Search only wiki pages
wm search query "query" --limit 50     # Max results
wm search retrieve "query"             # Context pack retrieval
wm search retrieve "query" --token-budget 4096  # With token limit
wm search resolve "id"                 # Resolve a page ID
```

### `wm page`
Wiki page CRUD operations.

```bash
wm page create tasks/my-task "Title" --content "Body"            # Create a task
wm page create concepts/my-concept "Title" --content "Body"      # Create a concept
wm page create memory/my-mem "Title" --content "Body"            # Create a memory entry
wm page get wiki:tasks:my-task --json                            # Get page by ID
wm page list                                                     # List all pages
wm page list --json                                              # JSON output
wm page link wiki:tasks:a wiki:tasks:b --edge-type depends_on    # Link pages
wm page update wiki:tasks:my-task --status done                  # Update fields
wm page delete wiki:tasks:my-task                                # Delete page
```

### `wm task`
Task board and task operations.

```bash
wm task board --json                     # Task board with counts
wm task list --status todo               # List tasks by status
wm task list --assignee alice            # List tasks by assignee
```

### `wm graph`
Graph exploration and navigation.

```bash
wm graph stats                           # Node/edge counts
wm graph neighbors wiki:tasks:my-task    # Show connected pages
wm graph path wiki:a wiki:b              # Shortest path between pages
wm graph subgraph wiki:tasks:my-task     # Neighborhood subgraph
```

### `wm memory`
Memory entry operations (agent-written knowledge).

```bash
wm memory list                           # List project memory
wm memory list --layer session           # List session memory
wm memory list --status active           # Filter by status
wm memory get wiki:memory:abc123         # Get single entry
wm memory add "Title" --content "Text"   # Add memory entry
wm memory update wiki:memory:abc123 --title "New"  # Update
wm memory delete wiki:memory:abc123      # Delete
wm memory promote wiki:memory:abc123     # Promote to global
```

### `wm template`
Template listing and rendering.

```bash
wm template list                         # List all templates
wm template get my-template              # Get template details
wm template run my-template              # Render template with prompts
wm template create my-template           # Create a new template
```

### `wm code`
Code intelligence.

```bash
wm code.search "pattern"                 # Regex code search
wm code.symbols                          # Find symbol definitions
wm code.deps                             # Show import dependencies
```

### `wm decision`
Architectural Decision Records.

```bash
wm decision create 20260714-use-wire "Title" --context "..." --rationale "..." --outcome "..."
wm decision get 20260714-use-wire
```

### `wm time`
Time tracking.

```bash
wm time start wiki:tasks:my-task         # Start timer
wm time stop wiki:tasks:my-task          # Stop and record
wm time add wiki:tasks:my-task 2h30m     # Manually add time
wm time report                           # Time summary
```

### `wm version`
Version history (field-level change tracking).

```bash
wm version list wiki:tasks:my-task       # List versions
wm version get wiki:tasks:my-task v3     # Get specific version
wm version rollback wiki:tasks:my-task v1  # Rollback to version
```

### `wm index`
Search index management.

```bash
wm index rebuild                         # Full index rebuild
wm index embed                           # Build embeddings only
wm index status                          # Index state
```

### `wm lint`
Wiki quality checks.

```bash
wm lint check                            # Check for issues
wm lint fix                              # Auto-fix issues
```

### `wm validate`
Wiki validation.

```bash
wm validate                              # Validate wiki health
wm validate --json                       # JSON output
```

### `wm log`
Audit log queries.

```bash
wm log recent                            # Last 20 entries
wm log recent --count 100                # Last 100 entries
wm log since "2026-07-14T..."            # Entries after timestamp
wm log filter "auth"                     # Filter by text
```

### `wm source`
Source management (external content ingestion).

```bash
wm source list                           # List sources
wm source list --state done              # Filter by state
wm source status my-source               # Source details
wm source discover                       # Discover new sources
wm source remove my-source               # Remove source
```

### `wm model`
Embedding model management.

```bash
wm model list                            # List models
wm model status                          # Current model state
wm model download bge-small-en-v1.5      # Download model
wm model remove bge-small-en-v1.5       # Remove model
```

### `wm web`
Start the web UI server.

```bash
wm web                                   # Start on port 3000
wm web --port 8080                       # Custom port
wm web --project /path/to/project        # Specific project
```

### `wm migrate-memory`
One-time migration from old `.wm/memory/*.json` to `.wm/wiki/memory/*.md`.

```bash
wm migrate-memory                        # Migrate all memory entries
```

### `wm init`, `wm setup`, `wm agents`
See dedicated sections above.

### `wm version`
Print version information.

```bash
wm version                               # Show version
```
