# Skill Management

WM skills (step-by-step agent instructions) are embedded in the `wm-cli` binary.

## Canonical Location

All skills live in `apps/wm-core/src/embed_files/skills/<name>/SKILL.md`. Edit these files to modify skills.

```
apps/wm-core/src/embed_files/skills/
├── wm-commit/SKILL.md     # Conventional commits
├── wm-debug/SKILL.md       # Structured debugging
├── wm-doc/SKILL.md         # Documentation operations
├── wm-extract/SKILL.md     # Knowledge extraction
├── wm-flow/SKILL.md        # Spec/task wave orchestration
├── wm-go/SKILL.md          # Automated spec pipeline
├── wm-implement/SKILL.md   # Task implementation
├── wm-init/SKILL.md        # Session initialization
├── wm-plan/SKILL.md        # Task planning
├── wm-research/SKILL.md    # Codebase research
├── wm-review/SKILL.md      # Code review
├── wm-spec/SKILL.md        # Spec creation
├── wm-template/SKILL.md    # Code generation templates
├── wm-validate/SKILL.md    # Wiki validation
└── wm-verify/SKILL.md      # SDD verification
```

## Syncing Skills

After editing a canonical skill, sync it to platform directories:

```bash
wm agents --sync
```

This copies skills to the configured platform directories:
- `.opencode/skills/` (OpenCode)
- `.claude/skills/` (Claude Code)
- `.kiro/skills/` (Kiro)
- `.codex/skills/` (Codex)
- `.agent/skills/` and `.agents/skills/` (generic agents)
- `.gemini/antigravity/skills/` (Antigravity)

## How It Works

Skills are compiled into the `wm-cli` binary via `rust-embed` (the `EmbeddedFiles` struct in `apps/wm-core/src/embed_files.rs`). At startup they are available for sync; `wm agents --sync` distributes them to all configured platform skill directories.

## Adding a New Skill

1. Create `apps/wm-core/src/embed_files/skills/<name>/SKILL.md`
2. Register it with the skill loader in `apps/wm-core/src/skill.rs`
3. Run `wm agents --sync` to distribute it
4. Rebuild: `cargo build -p wm-cli`
