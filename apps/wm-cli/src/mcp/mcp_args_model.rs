use clap::Args;
use std::path::PathBuf;

use super::mcp_transport_model::McpTransport;

#[derive(Args)]
pub struct McpArgs {
    #[arg(long)]
    pub project: Option<PathBuf>,

    #[arg(long, value_enum, default_value_t = McpTransport::Stdio)]
    pub transport: McpTransport,
}
