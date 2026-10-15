use clap::Subcommand;

#[derive(Subcommand)]
pub enum LintAction {
    Check {
        #[arg(long)]
        json: bool,
    },

    Fix {
        #[arg(long)]
        json: bool,
    },
}
