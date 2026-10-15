use clap::Subcommand;

use super::agents_args_model::AgentsArgs;
use super::config_action_model::ConfigAction;
use super::decide_args_model::DecideArgs;
use super::graph_action_model::GraphAction;
use super::health_action_model::HealthAction;
use super::index_action_model::IndexAction;
use super::init_args_model::InitArgs;
use super::lint_action_model::LintAction;
use super::log_action_model::LogAction;
use super::mcp_args_model::McpArgs;
use super::migrate_memory_args_model::MigrateMemoryArgs;
use super::model_action_model::ModelAction;
use super::page_action_model::PageAction;
use super::search_action_model::SearchAction;
use super::setup_args_model::SetupArgs;
use super::source_action_model::SourceAction;
use super::status_args_model::StatusArgs;
use super::task_action_model::TaskAction;
use super::time_action_model::TimeAction;
use super::validate_args_model::ValidateArgs;
use super::version_args_model::VersionArgs;

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
