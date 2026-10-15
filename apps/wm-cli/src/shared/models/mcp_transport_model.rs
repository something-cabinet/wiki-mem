#[derive(Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum McpTransport {
    #[default]
    Stdio,
}
