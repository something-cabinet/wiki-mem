use clap::Parser;

use super::commands_model::Commands;

#[derive(Parser)]
#[command(name = "wm", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}
