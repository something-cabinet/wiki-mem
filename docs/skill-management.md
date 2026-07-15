# Skill Management

WM skills (step-by-step agent instructions) are embedded in the `wm-cli` binary.

## Canonical Location

All skills live in `apps/wm-core/src/skills/<name>/SKILL.md`. Edit these files to modify skills.

```
apps/wm-core/src/skills/
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

This copies skills to:
- `.claude/skills/` (Claude Code)
- `.agents/skills/` (Generic agents)
- Other platform directories as configured

## How It Works

Skills are compiled into the `wm-cli` binary via `rust-embed`. At startup, they are auto-synced to the `.wm/skills/` directory. The `wm agents --sync` command extends this to all configured platform skill directories.

## Adding a New Skill

1. Create `apps/wm-core/src/skills/<name>/SKILL.md`
2. Add it to the skill loader in `apps/wm-core/src/skill.rs`
3. Run `wm agents --sync` to distribute it
4. Rebuild: `cargo build -p wm-cli`
