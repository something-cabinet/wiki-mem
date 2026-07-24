use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/shim_templates/"]
pub struct ShimTemplates;
