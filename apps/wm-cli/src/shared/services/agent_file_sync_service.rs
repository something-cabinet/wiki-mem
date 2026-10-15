use std::collections::HashSet;
use std::path::{Path, PathBuf};

const DEFAULT_PLATFORMS: [&str; 7] = [
    "claude", "opencode", "kiro", "gemini", "copilot", "agents", "reasonix",
];

const PLATFORM_SHIMS: [(&str, &str); 8] = [
    ("claude", "CLAUDE.md"),
    ("codex", "CLAUDE.md"),
    ("opencode", "AGENTS.md"),
    ("kiro", "AGENTS.md"),
    ("agents", "AGENTS.md"),
    ("gemini", "GEMINI.md"),
    ("copilot", ".github/copilot-instructions.md"),
    ("reasonix", "REASONIX.md"),
];

const SHIM_TEMPLATES: [(&str, &str); 6] = [
    ("CLAUDE.md", "shims/CLAUDE.md"),
    ("AGENTS.md", "shims/AGENTS.md"),
    ("GEMINI.md", "shims/GEMINI.md"),
    (
        ".github/copilot-instructions.md",
        "shims/copilot-instructions.md",
    ),
    ("REASONIX.md", "shims/REASONIX.md"),
    ("OPENCODE.md", "shims/OPENCODE.md"),
];

pub fn sync_agent_files(
    root: &Path,
    platforms: &[String],
    _force: bool,
) -> Result<(), anyhow::Error> {
    let targets = target_platforms(platforms);
    let mut written = HashSet::new();
    for platform in &targets {
        write_shim(root, platform, &mut written)?;
    }
    if targets.contains(&"opencode") {
        write_opencode_shim(root, &mut written);
    }
    Ok(())
}

fn target_platforms(platforms: &[String]) -> Vec<&str> {
    if platforms.is_empty() {
        return DEFAULT_PLATFORMS.to_vec();
    }
    platforms.iter().map(String::as_str).collect()
}

fn write_shim(
    root: &Path,
    platform: &str,
    written: &mut HashSet<String>,
) -> Result<(), anyhow::Error> {
    let Some((output_filename, template_key)) = shim_target(platform) else {
        return Ok(());
    };
    let content = resolve_template(template_key)?;
    let path = shim_path(root, output_filename);
    if written.insert(output_filename.to_string()) {
        std::fs::write(&path, content)?;
        println!("  {} — agent instruction file generated", output_filename);
        return Ok(());
    }
    announce_shared_shim(output_filename, platform);
    Ok(())
}

fn shim_target(platform: &str) -> Option<(&'static str, &'static str)> {
    let Some(output_filename) = compatible_shim(platform) else {
        warn_unknown_platform(platform);
        return None;
    };
    let template_key = shim_template(output_filename)?;
    Some((output_filename, template_key))
}

fn compatible_shim(platform: &str) -> Option<&'static str> {
    PLATFORM_SHIMS
        .iter()
        .find(|(name, _)| *name == platform)
        .map(|(_, shim)| *shim)
}

fn shim_template(output_filename: &str) -> Option<&'static str> {
    SHIM_TEMPLATES
        .iter()
        .find(|(shim, _)| *shim == output_filename)
        .map(|(_, template)| *template)
}

fn resolve_template(template_key: &str) -> Result<String, anyhow::Error> {
    wm_core::embed_files::EmbeddedFiles::get(template_key)
        .and_then(|f| std::str::from_utf8(f.data.as_ref()).ok().map(String::from))
        .ok_or_else(|| anyhow::anyhow!("Embedded shim template not found: {}", template_key))
}

fn shim_path(root: &Path, output_filename: &str) -> PathBuf {
    if output_filename.starts_with(".github") {
        let dir = root.join(".github");
        std::fs::create_dir_all(&dir).ok();
        return dir.join("copilot-instructions.md");
    }
    root.join(output_filename)
}

fn write_opencode_shim(root: &Path, written: &mut HashSet<String>) {
    let Some(content) = opencode_shim_content() else {
        return;
    };
    if std::fs::write(root.join("OPENCODE.md"), &content).is_ok()
        && !written.contains("OPENCODE.md")
    {
        written.insert("OPENCODE.md".to_string());
        println!("  OPENCODE.md — agent instruction file generated");
    }
}

fn opencode_shim_content() -> Option<String> {
    let file = wm_core::embed_files::EmbeddedFiles::get("shims/OPENCODE.md")?;
    std::str::from_utf8(file.data.as_ref())
        .ok()
        .map(String::from)
}

fn warn_unknown_platform(platform: &str) {
    eprintln!(
        "Unknown platform: {}. Use `wm init --platform <name>` for MCP config.",
        platform
    );
}

fn announce_shared_shim(output_filename: &str, platform: &str) {
    println!(
        "  {} — also handled by {} platform (same file)",
        output_filename, platform
    );
}

pub fn sync_skills_to(platform_skills_dir: &Path) -> Result<(), anyhow::Error> {
    let skills = wm_core::skill::load_embedded_skills();
    if skills.is_empty() {
        return Ok(());
    }
    std::fs::create_dir_all(platform_skills_dir)?;
    for skill in &skills {
        let skill_subdir = platform_skills_dir.join(&skill.name);
        std::fs::create_dir_all(&skill_subdir)?;

        let embed_path = format!("skills/{}/SKILL.md", skill.name);
        if let Some(file) = wm_core::embed_files::EmbeddedFiles::get(&embed_path) {
            std::fs::write(skill_subdir.join("SKILL.md"), &file.data)?;
        }
    }
    Ok(())
}
