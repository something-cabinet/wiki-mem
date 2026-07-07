# Knowns — Reference

## GitHub
- Repo: https://github.com/knowns-dev/knowns
- Docs: https://github.com/knowns-dev/knowns/tree/main/.knowns/docs
- opencode.json: https://github.com/knowns-dev/knowns/blob/main/opencode.json

## MCP Configuration
Knowns runs as an MCP server via: knowns mcp --stdio
OpenCode config entry: {"knowns": {"command": ["knowns", "mcp", "--stdio"], "enabled": true, "type": "local"}}

## Architecture
- Language: Go (rewritten from previous version)
- CLI style: Bubble Tea TUI with --json/--plain/--no-pager flags
- Storage: Markdown files as canonical store
- Search: Keyword (BM25) + Semantic (embeddings)
- MCP: JSON-RPC 2.0 over stdio
- Skills: YAML frontmatter + markdown instructions

## Key Differences from WM Engine
| Feature | Knowns | WM Engine |
|---------|--------|-----------|
| Language | Go | Rust |
| Graph edges | Tag-based | Typed (extends, depends_on, etc.) |
| CLI TUI | Bubble Tea | Ratatui (scaffolded) |
| Search modes | Keyword + semantic | Keyword + semantic + hybrid (RRF) |
| Time tracking | Not built-in | Full (start/stop/add/report) |
| Source state machine | Not built-in | Full (add/process/complete) |
| Skill system | Trigger-based | Trigger-based + MCP registration |
