use crate::version::VersionArgs;
use crate::shared::traits::Command;

impl Command for VersionArgs {
    async fn run(self) -> Result<(), anyhow::Error> {
        print_version();
        Ok(())
    }
}

fn print_version() {
    println!("wm {}", env!("CARGO_PKG_VERSION"));
}
