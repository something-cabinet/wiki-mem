#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ExportFormat {
    Json,
    Graphml,
    Obsidian,
}
