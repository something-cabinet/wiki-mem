use rust_embed::RustEmbed;

/// Embedded shim template files for agent instruction files (AGENTS.md, CLAUDE.md, etc.)
/// Generated at compile time from `src/shim_templates/`.
#[derive(RustEmbed)]
#[folder = "src/shim_templates/"]
pub struct ShimTemplates;
