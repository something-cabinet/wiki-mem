use crate::shared::models::SetupArgs;
use crate::shared::services::{setup_platform_mcp, sync_agent_files};
use crate::shared::traits::Command;
use wm_core::config;

const PLATFORM_ALL: &str = "all";
const ALL_PLATFORMS: [&str; 6] = [
    "opencode",
    "kiro",
    "claude",
    "codex",
    "cursor",
    "antigravity",
];
const SYNCABLE_PLATFORMS: [&str; 8] = [
    "opencode", "kiro", "claude", "codex", "gemini", "copilot", "agents", "reasonix",
];

impl Command for SetupArgs {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_setup(self)
    }
}

fn run_setup(args: SetupArgs) -> Result<(), anyhow::Error> {
    let root = config::detect_project_root().unwrap_or_else(|| std::env::current_dir().unwrap());
    let platforms = requested_platforms(&args.platform);
    let syncable = syncable_platforms(&platforms);
    sync_agent_files(&root, &syncable, false)?;
    for platform in &platforms {
        setup_platform_mcp(&root, platform)?;
        setup_query_before_grep(&root, platform, args.strict)?;
    }
    Ok(())
}

fn requested_platforms(platform: &str) -> Vec<String> {
    if platform == PLATFORM_ALL {
        return ALL_PLATFORMS.iter().map(|p| p.to_string()).collect();
    }
    vec![platform.to_owned()]
}

fn syncable_platforms(platforms: &[String]) -> Vec<String> {
    platforms
        .iter()
        .filter(|p| SYNCABLE_PLATFORMS.contains(&p.as_str()))
        .cloned()
        .collect()
}

fn setup_query_before_grep(
    root: &std::path::Path,
    platform: &str,
    strict: bool,
) -> Result<(), anyhow::Error> {
    let content = write_query_before_grep_hook(root, strict)?;
    match platform {
        "opencode" => hook_opencode(root, strict),
        "claude" | "codex" => hook_claude_codex(root, platform, strict),
        "kiro" => hook_kiro(root, strict),
        "cursor" => hook_cursor(root, &content, strict),
        "antigravity" => hook_antigravity(root, &content, strict),
        _ => Ok(()),
    }
}

fn write_query_before_grep_hook(
    root: &std::path::Path,
    strict: bool,
) -> Result<String, anyhow::Error> {
    let hook_path = root.join(wm_core::agent_hooks::QUERY_BEFORE_GREP_REL_PATH);
    if let Some(parent) = hook_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = wm_core::agent_hooks::query_before_grep_content(strict);
    std::fs::write(&hook_path, &content)?;
    Ok(content)
}

fn hook_opencode(root: &std::path::Path, strict: bool) -> Result<(), anyhow::Error> {
    let cfg_path = root.join("opencode.json");
    if let Some(existing) = read_json_file(&cfg_path) {
        let patched = wm_core::agent_hooks::opencode_with_hook(existing, strict);
        wm_core::platform_service::write_merged_json(&cfg_path, patched)?;
    }
    print_opencode_hook(strict);
    Ok(())
}

fn read_json_file(path: &std::path::Path) -> Option<serde_json::Value> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
}

fn print_opencode_hook(strict: bool) {
    println!(
        "  {} — query-before-grep instruction hooked into opencode.json",
        wm_core::agent_hooks::QUERY_BEFORE_GREP_REL_PATH
    );
    if strict {
        println!("    strict: read/grep/bash permission-gated (ask) — raw reads require approval");
    }
}

fn hook_claude_codex(
    root: &std::path::Path,
    platform: &str,
    strict: bool,
) -> Result<(), anyhow::Error> {
    append_instruction_import(root, "CLAUDE.md")?;
    if strict && platform == "claude" {
        write_claude_strict_settings(root)?;
    }
    println!(
        "  {} — query-before-grep instruction imported into CLAUDE.md",
        wm_core::agent_hooks::QUERY_BEFORE_GREP_REL_PATH
    );
    if strict && platform == "codex" {
        println!("    strict: instruction-only (codex has no per-tool permission rules)");
    }
    Ok(())
}

fn write_claude_strict_settings(root: &std::path::Path) -> Result<(), anyhow::Error> {
    let settings_dir = root.join(".claude");
    std::fs::create_dir_all(&settings_dir)?;
    wm_core::platform_service::write_merged_json(
        &settings_dir.join("settings.json"),
        wm_core::agent_hooks::claude_strict_settings(),
    )?;
    println!("  .claude/settings.json — strict: Read(//**)/Bash(*) permission-gated (ask)");
    Ok(())
}

fn hook_kiro(root: &std::path::Path, strict: bool) -> Result<(), anyhow::Error> {
    append_instruction_import(root, "AGENTS.md")?;
    println!(
        "  {} — query-before-grep instruction imported into AGENTS.md",
        wm_core::agent_hooks::QUERY_BEFORE_GREP_REL_PATH
    );
    if strict {
        println!("    strict: instruction-only (kiro has no permission rules)");
    }
    Ok(())
}

fn append_instruction_import(root: &std::path::Path, file: &str) -> Result<(), anyhow::Error> {
    let import = wm_core::agent_hooks::instruction_import_line();
    let target = root.join(file);
    if let Ok(existing) = std::fs::read_to_string(&target) {
        if !existing.contains(&import) {
            std::fs::write(&target, format!("{}\n\n{}", existing.trim_end(), import))?;
        }
    }
    Ok(())
}

fn hook_cursor(root: &std::path::Path, content: &str, strict: bool) -> Result<(), anyhow::Error> {
    let rules_dir = root.join(".cursor").join("rules");
    std::fs::create_dir_all(&rules_dir)?;
    let rule = format!(
        "---\ndescription: Query wm_graph/wm_search before raw file greps\n---\n\n{}",
        content
    );
    std::fs::write(rules_dir.join("query-before-grep.mdc"), rule)?;
    println!("  .cursor/rules/query-before-grep.mdc — query-before-grep cursor rule");
    if strict {
        println!("    strict: instruction-only (cursor has no permission rules)");
    }
    Ok(())
}

fn antigravity_skill_dir(root: &std::path::Path) -> std::path::PathBuf {
    root.join(".agents")
        .join("skills")
        .join("query-before-grep")
}

fn hook_antigravity(
    root: &std::path::Path,
    content: &str,
    strict: bool,
) -> Result<(), anyhow::Error> {
    let skill_dir = antigravity_skill_dir(root);
    std::fs::create_dir_all(&skill_dir)?;
    let skill = format!(
        "---\nname: query-before-grep\ndescription: Query wm_graph/wm_search before falling back to raw file greps\n---\n\n{}",
        content
    );
    std::fs::write(skill_dir.join("SKILL.md"), skill)?;
    println!("  .agents/skills/query-before-grep/SKILL.md — query-before-grep skill");
    if strict {
        println!("    strict: instruction-only (antigravity has no permission rules)");
    }
    Ok(())
}
