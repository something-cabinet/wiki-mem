use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/skills/"]
pub struct SkillAssets;
