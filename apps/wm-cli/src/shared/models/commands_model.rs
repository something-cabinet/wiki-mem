use clap::Subcommand;

use crate::agents::AgentsArgs;
use crate::config::ConfigAction;
use crate::decide::DecideArgs;
use crate::graph::GraphAction;
use crate::health::HealthAction;
use crate::index::IndexAction;
use crate::init::InitArgs;
use crate::lint::LintAction;
use crate::log::LogAction;
use crate::mcp::McpArgs;
use crate::migrate_memory::MigrateMemoryArgs;
use crate::model::ModelAction;
use crate::page::PageAction;
use crate::search::SearchAction;
use crate::setup::SetupArgs;
use crate::source::SourceAction;
use crate::status::StatusArgs;
use crate::task::TaskAction;
use crate::time::TimeAction;
use crate::validate::ValidateArgs;
use crate::version::VersionArgs;

#[derive(Subcommand)]
pub enum Commands {
    Init(InitArgs),

    Mcp(McpArgs),

    Setup(SetupArgs),

    Agents(AgentsArgs),

    Search {
        #[command(subcommand)]
        action: SearchAction,
    },

    Index {
        #[command(subcommand)]
        action: IndexAction,
    },

    Page {
        #[command(subcommand)]
        action: PageAction,
    },

    Graph {
        #[command(subcommand)]
        action: GraphAction,
    },

    Health {
        #[command(subcommand)]
        action: HealthAction,
    },

    Source {
        #[command(subcommand)]
        action: SourceAction,
    },

    Task {
        #[command(subcommand)]
        action: TaskAction,
    },

    Log {
        #[command(subcommand)]
        action: LogAction,
    },

    Lint {
        #[command(subcommand)]
        action: LintAction,
    },

    Validate(ValidateArgs),

    Time {
        #[command(subcommand)]
        action: TimeAction,
    },

    Model {
        #[command(subcommand)]
        action: ModelAction,
    },

    Decide(DecideArgs),

    Status(StatusArgs),

    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    #[command(alias = "--version")]
    Version(VersionArgs),

    MigrateMemory(MigrateMemoryArgs),
}
