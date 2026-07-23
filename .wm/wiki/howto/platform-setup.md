---
title: Platform Setup
type: howto
tags: [platform, setup, config, skills, mcp]
---

# Platform Setup

> Type: concept | Tags: [platform, setup, config, skills, mcp]

## Overview

Wiki Memory Engine supports multiple AI coding platforms through auto-generated instruction files, MCP configurations, and skill directories. The `wm init` command detects platform requirements from `config.json` and creates the appropriate files for each supported platform. This is modeled after Knowns' platform integration system.

## Supported Platforms

| Platform | MCP Config | Skill Dir | Instruction Files |
|----------|-----------|-----------|-------------------|
| Claude Code | `.mcp.json` | `.claude/skills/` | `CLAUDE.md` |
| OpenCode | `opencode.json` | `.opencode/skills/` | `OPENCODE.md`, `AGENTS.md` |
| Codex | `.codex/config.toml` | `.codex/skills/` | — |
| Kiro | `.kiro/settings/mcp.json` | `.kiro/skills/` | — |
| Antigravity | Global MCP config | `.agents/skills/` | `GEMINI.md` |
| Cursor | `.cursor/mcp.json` | `.cursor/skills/` | — |
| Copilot | `.github/copilot-instructions.md` | — | — |
| Generic agents | — | `.agent/skills/` | `AGENTS.md` |

## Technical Explanation

### MCP Configuration Files

Each platform requires a different MCP config format. WM generates these during `wm init`:

**Claude Code (`.mcp.json`):**
```json
{
  "mcpServers": {
    "wm-engine": {
      "command": "wm-cli",
      "args": ["serve"]
    }
  }
}
```

**OpenCode (`opencode.json`):**
```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "wm-engine": {
      "type": "local",
      "command": ["wm-cli", "serve"],
      "enabled": true
    }
  }
}
```

**Codex (`.codex/config.toml`):**
```toml
[mcp_servers.wm-engine]
command = "wm-cli"
args = ["serve"]
```

### Skill File Locations

WM auto-generates skill files from `.wm/skills/*.md` templates. During `wm init`, these skills are synchronized to platform-specific directories:

| Source | Target Platform | Target Directory |
|--------|----------------|------------------|
| `.wm/skills/wm-init/SKILL.md` | Claude Code | `.claude/skills/wm-init/SKILL.md` |
| `.wm/skills/wm-plan/SKILL.md` | OpenCode | `.opencode/skills/wm-plan/SKILL.md` |
| `.wm/skills/wm-commit/SKILL.md` | Codex | `.codex/skills/wm-commit/SKILL.md` |
| `.wm/skills/wm-implement/SKILL.md` | Kiro | `.kiro/skills/wm-implement/SKILL.md` |
| `.wm/skills/wm-extract/SKILL.md` | Antigravity | `.agents/skills/wm-extract/SKILL.md` |

The skill sync mapping is:
- `.claude/skills/` → Claude Code
- `.opencode/skills/` → OpenCode
- `.codex/skills/` → Codex
- `.agents/skills/` → Antigravity
- `.agent/skills/` → Generic agents (fallback)
- `.kiro/skills/` → Kiro

### Instruction Files

WM generates platform-specific instruction shims:

- **`CLAUDE.md`** — for Claude Code and Claude Desktop
- **`OPENCODE.md`** — for OpenCode
- **`GEMINI.md`** — for Gemini/Antigravity
- **`AGENTS.md`** — universal compatibility entrypoint (all platforms fall back to this)

Each file contains a copy of `.wm/AGENTS.md` content adapted for the platform's conventions. These files point the AI assistant to use WM MCP tools for project context.

### Config Paths

| Config File | Location | Purpose |
|------------|----------|---------|
| `.wm/config.json` | Project root | Project-level WM settings |
| `~/.wm/config.json` | User home | Global WM settings (model cache, defaults) |
| `.mcp.json` | Project root | Claude Code MCP config |
| `opencode.json` | Project root | OpenCode MCP config |
| `.codex/config.toml` | Project root | Codex MCP config |
| `.cursor/mcp.json` | Project root | Cursor MCP config |

### `wm init` Flow

1. Read `config.json` → `platforms` array
2. For each enabled platform:
   - Create MCP config file if not present
   - Sync skill files from `.wm/skills/` to platform skill dir
   - Generate platform-specific instruction file
3. Create `.wm/` directory structure
4. Generate `AGENTS.md` in `.wm/`
5. Create default skill templates
6. Create wiki subdirectories

### Platform-Specific Skill Directories

Skills are organized in `wm-core/src/skills/` as SKILL.md files:

```
wm-core/src/skills/
├── wm-extract/SKILL.md
├── wm-verify/SKILL.md
├── wm-template/SKILL.md
├── wm-spec/SKILL.md
├── wm-review/SKILL.md
├── wm-research/SKILL.md
├── wm-plan/SKILL.md
├── wm-init/SKILL.md
├── wm-implement/SKILL.md
├── wm-go/SKILL.md
├── wm-doc/SKILL.md
├── wm-debug/SKILL.md
└── wm-commit/SKILL.md
```

These are embedded into the binary via `rust-embed` and extracted to platform-specific skill directories (`.opencode/skills/`, `.codex/skills/`, `.agents/skills/`, `.claude/skills/`, `.kiro/skills/`, etc.) during `wm setup`.

## Configuration Reference

```json
// .wm/config.json
{
  "platforms": [
    "claude-code",
    "opencode",
    "codex",
    "kiro",
    "antigravity",
    "cursor",
    "copilot",
    "agents"
  ]
}
```

Each platform name maps to a specific set of generated files. Omitting a platform skips its file generation.

## Related Documents

- [ScoringConfig](./scoring-config.md) — config.json structure reference
- [Memory System](./memory-system.md) — `.wm/memory/` data format
- [Cross-Entity Search](./cross-entity-search.md) — how MCP clients use search