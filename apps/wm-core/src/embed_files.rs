use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/embed_files/"]
pub struct EmbeddedFiles;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_shims() {
        let shim =
            EmbeddedFiles::get("shims/OPENCODE.md").expect("shims/OPENCODE.md should be embedded");
        let content = std::str::from_utf8(&shim.data).unwrap();
        assert!(
            content.contains("OPENCODE"),
            "Should contain OPENCODE content"
        );
        println!("✓ shims/OPENCODE.md embedded OK");
    }

    #[test]
    fn test_embedded_skills() {
        let skill = EmbeddedFiles::get("skills/wm-init/SKILL.md")
            .expect("skills/wm-init/SKILL.md should be embedded");
        let content = std::str::from_utf8(&skill.data).unwrap();
        assert!(
            content.contains("wm-init"),
            "Should contain wm-init content"
        );
        println!("✓ skills/wm-init/SKILL.md embedded OK");
    }

    #[test]
    fn test_embedded_configs() {
        let config = EmbeddedFiles::get("configs/opencode.json")
            .expect("configs/opencode.json should be embedded");
        let content = std::str::from_utf8(&config.data).unwrap();
        assert!(
            content.contains("wm-cli"),
            "Should contain wm-cli in opencode.json"
        );
        println!("✓ configs/opencode.json embedded OK");

        let config = EmbeddedFiles::get("configs/dot_mcp.json")
            .expect("configs/dot_mcp.json should be embedded");
        let content = std::str::from_utf8(&config.data).unwrap();
        assert!(
            content.contains("wm-cli"),
            "Should contain wm-cli in dot_mcp.json"
        );
        println!("✓ configs/dot_mcp.json embedded OK");

        let config = EmbeddedFiles::get("configs/codex_config.toml")
            .expect("configs/codex_config.toml should be embedded");
        let content = std::str::from_utf8(&config.data).unwrap();
        assert!(
            content.contains("wm-cli"),
            "Should contain wm-cli in codex_config.toml"
        );
        println!("✓ configs/codex_config.toml embedded OK");
    }
}
