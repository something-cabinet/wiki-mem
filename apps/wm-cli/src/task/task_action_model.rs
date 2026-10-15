use clap::Subcommand;

#[derive(Subcommand)]
pub enum TaskAction {
    Board {
        #[arg(long)]
        json: bool,
    },
}
