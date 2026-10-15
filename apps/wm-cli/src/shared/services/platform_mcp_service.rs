use std::path::Path;

use super::agent_file_sync_service::sync_skills_to;

pub fn setup_platform_mcp(root: &Path, platform: &str) -> Result<(), anyhow::Error> {
    let resolved = resolve_mcp_binary();
    match platform {
        "opencode" => setup_opencode(root)?,
        "kiro" => setup_kiro(root)?,
        "claude" => setup_claude(root)?,
        "codex" => setup_codex(root, &resolved)?,
        "cursor" => setup_cursor(root)?,
        "antigravity" => setup_antigravity(root)?,
        _ => {}
    }
    Ok(())
}

fn setup_opencode(root: &Path) -> Result<(), anyhow::Error> {
    let cfg = root.join("opencode.json");
    write_embedded_json_config(&cfg, "configs/opencode.json")?;
    write_embedded_file(root, "shims/OPENCODE.md", "OPENCODE.md")?;
    sync_skills_to(&root.join(".opencode").join("skills"))?;
    println!(
        "  {} — OpenCode MCP config (+ skills synced to .opencode/skills/)",
        cfg.display()
    );
    Ok(())
}

fn setup_kiro(root: &Path) -> Result<(), anyhow::Error> {
    let cfg_dir = root.join(".kiro").join("settings");
    std::fs::create_dir_all(&cfg_dir).ok();
    let cfg = cfg_dir.join("mcp.json");
    write_embedded_json_config(&cfg, "configs/kiro_mcp.json")?;
    sync_skills_to(&root.join(".kiro").join("skills"))?;
    write_kiro_steering(root);
    println!("  {} — Kiro MCP config (+ skills, steering)", cfg.display());
    Ok(())
}

fn write_kiro_steering(root: &Path) {
    let steering_dir = root.join(".kiro").join("steering");
    std::fs::create_dir_all(&steering_dir).ok();
    if let Some(template) = wm_core::embed_files::EmbeddedFiles::get("steering/wiki-mem.md") {
        std::fs::write(steering_dir.join("wiki-mem.md"), &template.data).ok();
    }
}

fn setup_claude(root: &Path) -> Result<(), anyhow::Error> {
    let cfg_file = root.join(".mcp.json");
    write_embedded_json_config(&cfg_file, "configs/dot_mcp.json")?;
    sync_skills_to(&root.join(".claude").join("skills"))?;
    println!(
        "  {} — Claude MCP config (+ skills synced to .claude/skills/)",
        cfg_file.display()
    );
    Ok(())
}

fn setup_codex(root: &Path, resolved: &str) -> Result<(), anyhow::Error> {
    let dir = root.join(".codex");
    std::fs::create_dir_all(&dir).ok();
    let cfg_file = dir.join("config.toml");
    wm_core::platform_service::write_toml_config(&cfg_file, resolved)?;
    sync_skills_to(&root.join(".codex").join("skills"))?;
    println!(
        "  {} — Codex MCP config (TOML) (+ skills synced to .codex/skills/)",
        cfg_file.display()
    );
    Ok(())
}

fn setup_cursor(root: &Path) -> Result<(), anyhow::Error> {
    let cfg_dir = root.join(".cursor");
    std::fs::create_dir_all(&cfg_dir).ok();
    let cfg = cfg_dir.join("mcp.json");
    write_embedded_json_config(&cfg, "configs/cursor_mcp.json")?;
    sync_skills_to(&root.join(".agent").join("skills"))?;
    println!("  {} — Cursor MCP config (+ skills synced)", cfg.display());
    Ok(())
}

fn setup_antigravity(root: &Path) -> Result<(), anyhow::Error> {
    let gemini_dir = root.join(".gemini").join("antigravity");
    std::fs::create_dir_all(&gemini_dir).ok();
    let cfg = gemini_dir.join("mcp_config.json");
    write_embedded_json_config(&cfg, "configs/antigravity_mcp.json")?;
    sync_skills_to(&root.join(".agents").join("skills"))?;
    println!(
        "  {} — Antigravity MCP config (+ skills synced to .agents/skills/)",
        cfg.display()
    );
    Ok(())
}

fn write_embedded_json_config(cfg: &Path, embedded_key: &str) -> Result<(), anyhow::Error> {
    let embedded = read_embedded_json(embedded_key)?;
    wm_core::platform_service::write_merged_json(cfg, patch_mcp_command(embedded))?;
    Ok(())
}

fn read_embedded_json(key: &str) -> Result<serde_json::Value, anyhow::Error> {
    let file = wm_core::embed_files::EmbeddedFiles::get(key)
        .ok_or_else(|| anyhow::anyhow!("Embedded config not found: {}", key))?;
    Ok(serde_json::from_slice(&file.data)?)
}

fn write_embedded_file(root: &Path, key: &str, filename: &str) -> Result<(), anyhow::Error> {
    let Some(file) = wm_core::embed_files::EmbeddedFiles::get(key) else {
        return Ok(());
    };
    let Ok(content) = std::str::from_utf8(file.data.as_ref()) else {
        return Ok(());
    };
    std::fs::write(root.join(filename), content)?;
    Ok(())
}

fn resolve_mcp_binary() -> String {
    "wm-cli".into()
}

fn patch_mcp_command(mut cfg: serde_json::Value) -> serde_json::Value {
    let resolved = resolve_mcp_binary();
    if patch_array_form(&mut cfg, &resolved) {
        return cfg;
    }
    patch_string_form(&mut cfg, &resolved);
    cfg
}

fn patch_array_form(cfg: &mut serde_json::Value, resolved: &str) -> bool {
    let Some(command) = cfg
        .pointer_mut("/mcp/wm/command")
        .and_then(|v| v.as_array_mut())
    else {
        return false;
    };
    if let Some(first) = command.first_mut() {
        *first = serde_json::Value::String(resolved.to_string());
    }
    true
}

fn patch_string_form(cfg: &mut serde_json::Value, resolved: &str) {
    let Some(command) = cfg.pointer_mut("/mcpServers/wm/command") else {
        return;
    };
    if command.is_string() {
        *command = serde_json::Value::String(resolved.to_string());
    }
}
