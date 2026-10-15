use clap::Subcommand;
use std::path::PathBuf;

use super::export_format_model::ExportFormat;

#[derive(Subcommand)]
pub enum GraphAction {
    Neighbors {
        id: String,
        #[arg(long)]
        query: Option<String>,
        #[arg(long)]
        json: bool,
    },

    Path {
        start: String,
        end: String,
        #[arg(long)]
        max_depth: Option<usize>,
        #[arg(long)]
        json: bool,
    },

    Subgraph {
        center: String,
        #[arg(long)]
        depth: Option<usize>,
        #[arg(long)]
        json: bool,
    },

    Affected {
        node: String,
        #[arg(long)]
        max_depth: Option<usize>,
        #[arg(long)]
        json: bool,
    },

    Stats {
        #[arg(long)]
        json: bool,
    },

    Export {
        format: ExportFormat,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}
