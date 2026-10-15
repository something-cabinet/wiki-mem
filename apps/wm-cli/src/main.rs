use clap::{CommandFactory, Parser};

use shared::helpers::setup_logging;
use shared::models::{Cli, Commands};
use shared::traits::Command;

mod mcp_server;

mod constants;
mod models;
mod services;
mod shared;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    setup_logging();
    let cli = Cli::parse();

    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            Cli::command().print_help()?;
            println!();
            return Ok(());
        }
    };

    match command {
        Commands::Init(args) => args.run().await?,
        Commands::Mcp(args) => args.run().await?,
        Commands::Setup(args) => args.run().await?,
        Commands::Agents(args) => args.run().await?,
        Commands::Search { action } => action.run().await?,
        Commands::Page { action } => action.run().await?,
        Commands::Graph { action } => action.run().await?,
        Commands::Health { action } => action.run().await?,
        Commands::Source { action } => action.run().await?,
        Commands::Task { action } => action.run().await?,
        Commands::Log { action } => action.run().await?,
        Commands::Lint { action } => action.run().await?,
        Commands::Validate(args) => args.run().await?,
        Commands::Index { action } => action.run().await?,
        Commands::Time { action } => action.run().await?,
        Commands::Model { action } => action.run().await?,
        Commands::Decide(args) => args.run().await?,
        Commands::Status(args) => args.run().await?,
        Commands::Config { action } => action.run().await?,
        Commands::Version(args) => args.run().await?,
        Commands::MigrateMemory(args) => args.run().await?,
    }

    Ok(())
}
